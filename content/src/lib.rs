#![cfg(target_arch = "wasm32")]

use std::collections::{HashMap, HashSet};

use futures_util::{
    stream::{AbortHandle, AbortRegistration, Abortable},
    StreamExt, TryStreamExt,
};

use linked_data::{comments::Comment, identity::Identity, types::IPNSAddress};

use utils::{
    defluencer::ChannelContext, follows::get_follow_list, ipfs::IPFSContext, timestamp_to_datetime,
};

use yew::{platform::spawn_local, prelude::*};

use cid::Cid;

use gloo_console::{error, info};

use components::{navbar::NavigationBar, pure::Content};

use ipfs_api::IpfsService;

use defluencer::Defluencer;

use defluencer::crypto::signed_link::SignedLink;

use linked_data::media::Media;

use ybc::{Container, Section};

use components::searching::Searching;

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

        let (context, _) = ctx
            .link()
            .context::<IPFSContext>(Callback::noop())
            .expect("IPFS Context");

        let ipfs = context.client;

        spawn_local(get_content(
            ipfs.clone(),
            ctx.link().callback(Msg::Media),
            ctx.props().cid,
        ));

        let identity_cb = ctx.link().callback(Msg::Identity);

        let mut follows = get_follow_list();

        if let Some((context, _)) = ctx.link().context::<ChannelContext>(Callback::noop()) {
            follows.insert(context.channel.get_address());
        }

        let crawl_cb = ctx.link().callback(Msg::Crawl);
        let (crawl_handle, regis) = AbortHandle::new_pair();

        spawn_local(web_crawl(ipfs, follows, crawl_cb, regis));

        let comment_cb = ctx.link().callback(Msg::Comment);

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
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        #[cfg(debug_assertions)]
        info!("Content Page Update");

        let (context, _) = ctx
            .link()
            .context::<IPFSContext>(Callback::noop())
            .expect("IPFS Context");

        let ipfs = context.client;

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
                spawn_local(get_identity(
                    ipfs,
                    media.identity().link,
                    self.identity_cb.clone(),
                ));

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

                spawn_local(get_identity(
                    ipfs,
                    comment.identity.link,
                    self.identity_cb.clone(),
                ));

                self.comments.insert(cid, comment);

                true
            }
            Msg::Identity((cid, identity)) => {
                if self.identities.insert(cid, identity).is_some() {
                    return false;
                }

                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        #[cfg(debug_assertions)]
        info!("Content Page View");

        let content = match self.media.as_ref() {
            Some(media) => match self.identities.get(&media.identity().link) {
                Some(identity) => {
                    html! { <Content key={ctx.props().cid.to_string()} cid={ctx.props().cid} media={media.clone()} identity={identity.clone() } /> }
                }
                None => html! { <Searching /> },
            },
            None => html! { <Searching /> },
        };

        html! {
        <>
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
        </>
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
                if origin != comment.origin {
                    return None;
                }

                let identity = match self.identities.get(&comment.identity.link) {
                    Some(id) => id.clone(),
                    None => return None,
                };

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

    /* fn render_microblog(&self, ctx: &Context<Self>, blog: &MicroPost, identity: &Identity) -> Html {
        html! {
        <Box>
            <Level>
                <LevelLeft>
                    <LevelItem>
                        <Identification key={blog.identity.link.to_string()} cid={blog.identity.link} addr={self.addr.clone()} />
                    </LevelItem>
                    <LevelItem>
                        <span class="icon-text">
                            <span class="icon"><i class="fas fa-clock"></i></span>
                            <span> { &self.dt } </span>
                        </span>
                    </LevelItem>
                </LevelLeft>
                <LevelRight>
                    <small>{ ctx.props().cid.to_string() }</small>
                </LevelRight>
            </Level>
            <ybc::Content>
                { &blog.content }
            </ybc::Content>
            <Level>
                <LevelLeft>
                    <LevelItem>
                        <CommentButton cid={ctx.props().cid} callback={self.comment_cb.clone()} />
                    </LevelItem>
                    <LevelItem>
                        <ShareButton cid={ctx.props().cid} />
                    </LevelItem>
                </LevelLeft>
                <LevelRight>
                    <DagExplorer key={ctx.props().cid.to_string()} cid={ctx.props().cid} />
                </LevelRight>
            </Level>
        </Box>
        }
    } */

    /* fn render_article(&self, ctx: &Context<Self>, article: &FullPost, identity: &Identity) -> Html {
        html! {
        <Box>
            <Title>
                { &article.title }
            </Title>
            <ybc::Image size={ImageSize::Is16by9} >
                <Image cid={article.image.link} round=false />
            </ybc::Image>
            <Level>
                <LevelLeft>
                    <LevelItem>
                        <Identification key={article.identity.link.to_string()} cid={article.identity.link} addr={self.addr.clone()} />
                    </LevelItem>
                    <LevelItem>
                        <span class="icon-text">
                            <span class="icon"><i class="fas fa-clock"></i></span>
                            <span> { &self.dt } </span>
                        </span>
                    </LevelItem>
                </LevelLeft>
                <LevelRight>
                    <LevelItem>
                        <DagExplorer key={ctx.props().cid.to_string()} cid={ctx.props().cid} />
                    </LevelItem>
                </LevelRight>
            </Level>
            <ybc::Content>
                <Markdown cid={article.content.link} />
            </ybc::Content>
            <Level>
                <LevelLeft>
                    <LevelItem>
                        <CommentButton cid={ctx.props().cid} callback={self.comment_cb.clone()} />
                    </LevelItem>
                    <LevelItem>
                        <ShareButton cid={ctx.props().cid} />
                    </LevelItem>
                </LevelLeft>
            </Level>
        </Box>
        }
    } */

    /* fn render_video(&self, ctx: &Context<Self>, video: &Video, identity: &Identity) -> Html {
        html! {
        <Box>
            <Title>
                { &video.title }
            </Title>
            <Level>
                <LevelLeft>
                    <LevelItem>
                        <Identification key={video.identity.link.to_string()} cid={video.identity.link} addr={self.addr.clone()} />
                    </LevelItem>
                    <LevelItem>
                        <span class="icon-text">
                            <span class="icon"><i class="fas fa-clock"></i></span>
                            <span> { &self.dt } </span>
                        </span>
                    </LevelItem>
                </LevelLeft>
                <LevelRight>
                    <LevelItem>
                        <DagExplorer key={ctx.props().cid.to_string()} cid={ctx.props().cid} />
                    </LevelItem>
                </LevelRight>
            </Level>
            <VideoPlayer cid={ctx.props().cid} />
            <Level>
                <LevelLeft>
                    <LevelItem>
                        <CommentButton cid={ctx.props().cid} callback={self.comment_cb.clone()} />
                    </LevelItem>
                    <LevelItem>
                        <ShareButton cid={ctx.props().cid} />
                    </LevelItem>
                </LevelLeft>
            </Level>
        </Box>
        }
    } */

    /* fn render_comment(&self, ctx: &Context<Self>, comment: &Comment, identity: &Identity) -> Html {
        html! {
        <Box>
            <Level>
                <LevelLeft>
                    <LevelItem>
                        <Identification key={comment.identity.link.to_string()} cid={comment.identity.link} addr={self.addr.clone()} />
                    </LevelItem>
                    <LevelItem>
                        <span class="icon-text">
                            <span class="icon"><i class="fas fa-clock"></i></span>
                            <span> { &self.dt } </span>
                        </span>
                    </LevelItem>
                </LevelLeft>
                <LevelRight>
                    <small>{ ctx.props().cid.to_string() }</small>
                </LevelRight>
            </Level>
            <ybc::Content>
                { &comment.text }
            </ybc::Content>
            <Level>
                <LevelLeft>
                    <LevelItem>
                        <CommentButton cid={ctx.props().cid} callback={self.comment_cb.clone()} />
                    </LevelItem>
                    <LevelItem>
                        <ShareButton cid={ctx.props().cid} />
                    </LevelItem>
                </LevelLeft>
                <LevelRight>
                    <DagExplorer key={ctx.props().cid.to_string()} cid={ctx.props().cid} />
                </LevelRight>
            </Level>
        </Box>
        }
    } */
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
        error!("Content Verification Failed!");
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
    let defluencer = Defluencer::new(ipfs);

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
    let defluencer = Defluencer::new(ipfs.clone());

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

async fn get_identity(ipfs: IpfsService, cid: Cid, callback: Callback<(Cid, Identity)>) {
    match ipfs.dag_get::<&str, Identity>(cid, None).await {
        Ok(dag) => callback.emit((cid, dag)),
        Err(e) => error!(&format!("{:#?}", e)),
    }
}
