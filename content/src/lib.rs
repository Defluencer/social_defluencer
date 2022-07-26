#![cfg(target_arch = "wasm32")]

use std::collections::{HashMap, HashSet};

use futures_util::{
    stream::{AbortHandle, AbortRegistration, Abortable},
    StreamExt, TryStreamExt,
};

use utils::{
    commentary::CommentaryContext, defluencer::ChannelContext, ipfs::IPFSContext,
    subscriptions::get_sub_list, timestamp_to_datetime,
};

use yew::{platform::spawn_local, prelude::*};

use cid::Cid;

use gloo_console::error;

#[cfg(debug_assertions)]
use gloo_console::info;

use components::pure::{Content, NavigationBar, Searching};

use ipfs_api::IpfsService;

use defluencer::{crypto::signed_link::SignedLink, Defluencer};

use linked_data::{identity::Identity, media::comments::Comment, media::Media, types::IPNSAddress};

use ybc::{Container, Section};

#[derive(Properties, PartialEq)]
pub struct Props {
    /// Signed Link to Media Cid
    pub cid: Cid,
}

/// social.defluencer.eth/#/content/<CID_HERE>
///
/// Page displaying specific content & comments
pub struct ContentPage {
    media: Option<Media>,

    dt: String,

    addr: String,

    indexes: HashSet<Cid>,
    crawl_handle: AbortHandle,

    comment_cb: Callback<(Cid, Comment)>,
    comments: HashMap<Cid, Comment>,

    identity_cb: Callback<(Cid, Identity)>,
    identities: HashMap<Cid, Identity>,

    commentary: CommentaryContext,
}

pub enum Msg {
    Crawl(Cid),
    Media((Media, String)),
    Comment((Cid, Comment)),
    Identity((Cid, Identity)),
}

impl Component for ContentPage {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        #[cfg(debug_assertions)]
        info!("Content Page Create");

        let identity_cb = ctx.link().callback(Msg::Identity);

        let mut subcriptions = get_sub_list();

        if let Some((context, _)) = ctx.link().context::<ChannelContext>(Callback::noop()) {
            subcriptions.insert(context.channel.get_address());
        }

        let crawl_cb = ctx.link().callback(Msg::Crawl);
        let (crawl_handle, regis) = AbortHandle::new_pair();

        let comment_cb = ctx.link().callback(Msg::Comment);

        let commentary = CommentaryContext {
            callback: comment_cb.clone(),
        };

        if let Some((context, _)) = ctx.link().context::<IPFSContext>(Callback::noop()) {
            let ipfs = context.client;

            spawn_local(get_content(
                ipfs.clone(),
                ctx.link().callback(Msg::Media),
                ctx.props().cid,
            ));

            spawn_local(web_crawl(ipfs, subcriptions, crawl_cb, regis));
        }

        Self {
            media: None,
            dt: String::new(),
            addr: String::new(),

            indexes: HashSet::default(),
            crawl_handle,

            comment_cb,
            comments: HashMap::default(),

            identity_cb,
            identities: HashMap::default(),

            commentary,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        #[cfg(debug_assertions)]
        info!("Content Page Update");

        let ipfs = match ctx.link().context::<IPFSContext>(Callback::noop()) {
            Some((context, _)) => context.client,
            None => return false,
        };

        match msg {
            Msg::Crawl(index) => {
                if !self.indexes.insert(index) {
                    return false;
                }

                spawn_local(stream_comments(
                    ipfs,
                    index,
                    ctx.props().cid,
                    self.comment_cb.clone(),
                ));

                false
            }
            Msg::Media((media, addr)) => {
                if !self.identities.contains_key(&media.identity().link) {
                    spawn_local(utils::r#async::dag_get(
                        ipfs,
                        media.identity().link,
                        self.identity_cb.clone(),
                    ));
                }

                self.dt = timestamp_to_datetime(media.user_timestamp());
                self.media = Some(media);
                self.addr = addr;

                true
            }
            Msg::Comment((cid, comment)) => {
                if self.comments.contains_key(&cid) {
                    return false;
                }

                for index in self.indexes.iter() {
                    spawn_local(stream_comments(
                        ipfs.clone(),
                        *index,
                        cid,
                        self.comment_cb.clone(),
                    ));
                }

                if !self.identities.contains_key(&comment.identity.link) {
                    spawn_local(utils::r#async::dag_get(
                        ipfs,
                        comment.identity.link,
                        self.identity_cb.clone(),
                    ));
                }

                self.comments.insert(cid, comment);

                true
            }
            Msg::Identity((cid, identity)) => self.identities.insert(cid, identity).is_none(),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        #[cfg(debug_assertions)]
        info!("Content Page View");

        let mut content = html! { <Searching /> };

        if let Some(media) = self.media.as_ref() {
            if let Some(identity) = self.identities.get(&media.identity().link) {
                let mut verified = false;

                if let Some(eth_addr) = &identity.eth_addr {
                    if eth_addr == &self.addr {
                        verified = true;
                    }
                }

                content = html! { <Content key={ctx.props().cid.to_string()} cid={ctx.props().cid} media={media.clone()} identity={identity.clone()} {verified} /> };
            }
        }

        html! {
        <ContextProvider<CommentaryContext> context={self.commentary.clone()} >
            <NavigationBar />
            <Section>
                <Container>
                    { content }
                </Container>
            </Section>
            <Section>
                <Container>
                    { self.render_comments(ctx.props().cid)}
                </Container>
            </Section>
        </ContextProvider<CommentaryContext>>
        }
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        #[cfg(debug_assertions)]
        info!("Content Page Destroy");

        self.crawl_handle.abort();
    }
}

impl ContentPage {
    /// Recursively render all comments
    fn render_comments(&self, origin: Cid) -> Html {
        self.comments
            .iter()
            .filter_map(|(cid, comment)| {
                let comment_origin = comment.origin?;

                if origin != comment_origin {
                    return None;
                }

                let identity = self.identities.get(&comment.identity.link)?;
                let identity = identity.clone();

                let cid = *cid;
                let media = Media::Comment(comment.clone());

                return Some(html! {
                    <components::pure::Content key={cid.to_string()} {cid} {media} {identity} >
                        { self.render_comments(cid) }
                    </components::pure::Content>
                });
            })
            .collect::<Html>()
    }
}

async fn get_content(ipfs: IpfsService, callback: Callback<(Media, String)>, cid: Cid) {
    let signed_link = match ipfs.dag_get::<&str, SignedLink>(cid, None).await {
        Ok(dag) => dag,
        Err(e) => {
            error!(&format!("{:#?}", e));
            return;
        }
    };

    if !signed_link.verify() {
        error!("Content Signature Verification Failed!");
        return;
    }

    let addr = signed_link.get_address();

    let media = match ipfs
        .dag_get::<&str, Media>(signed_link.link.link, None)
        .await
    {
        Ok(dag) => dag,
        Err(e) => {
            error!(&format!("{:#?}", e));
            return;
        }
    };

    callback.emit((media, addr));
}

async fn web_crawl(
    ipfs: IpfsService,
    follows: HashSet<IPNSAddress>,
    callback: Callback<Cid>,
    regis: AbortRegistration,
) {
    let defluencer = Defluencer::from(ipfs);

    let stream = defluencer
        .streaming_web_crawl(follows.into_iter())
        .try_filter_map(|(_, channel)| async move { Ok(channel.comment_index) });

    let stream = Abortable::new(stream, regis);

    futures_util::pin_mut!(stream);

    while let Some(result) = stream.next().await {
        match result {
            Ok(ipld) => callback.emit(ipld.link),
            Err(e) => error!(&format!("{:#?}", e)),
        }
    }
}

async fn stream_comments(
    ipfs: IpfsService,
    index: Cid,
    content_cid: Cid,
    callback: Callback<(Cid, Comment)>,
) {
    let defluencer = Defluencer::from(ipfs.clone());

    let stream = defluencer
        .stream_content_comments(index.into(), content_cid)
        .map_ok(|cid| {
            let ipfs = ipfs.clone();

            async move {
                match ipfs.dag_get::<&str, Comment>(cid, Some("/link")).await {
                    Ok(dag) => Ok((cid, dag)),
                    Err(e) => Err(e.into()),
                }
            }
        })
        .try_buffer_unordered(10);

    futures_util::pin_mut!(stream);

    while let Some(result) = stream.next().await {
        match result {
            Ok(tuple) => callback.emit(tuple),
            Err(e) => error!(&format!("{:#?}", e)),
        }
    }
}
