#![cfg(target_arch = "wasm32")]

use std::collections::HashSet;

use defluencer::Defluencer;
use futures_util::{
    stream::{AbortHandle, AbortRegistration, Abortable},
    StreamExt, TryStreamExt,
};
use ipfs_api::IpfsService;
use linked_data::{identity::Identity, types::IPNSAddress};

use utils::{defluencer::ChannelContext, follows::get_follow_list, timestamp_to_datetime};

use cid::Cid;

use utils::ipfs::IPFSContext;

use yew::{platform::spawn_local, prelude::*};

use gloo_console::error;

use ybc::{
    Block, Content, ImageSize, Level, LevelItem, LevelLeft, Media, MediaContent, MediaLeft,
    MediaRight,
};

use yew_router::prelude::Link;

use crate::{
    comment_button::CommentButton, dag_explorer::DagExplorer, navbar::Route, pure::IPFSImage,
    searching::Searching, share_button::ShareButton,
};

#[derive(Properties, PartialEq)]
pub struct Props {
    /// Signed link to media Cid
    pub cid: Cid,
}

pub struct Comment {
    dt: String,
    comment: Option<linked_data::comments::Comment>,

    identity: Option<Identity>,

    handle: AbortHandle,

    reply: Option<Cid>,
}

pub enum Msg {
    Data(linked_data::comments::Comment),
    Identity(Identity),
    Reply(Cid),
}

impl Component for Comment {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let (context, _) = ctx
            .link()
            .context::<IPFSContext>(Callback::noop())
            .expect("IPFS Context");

        spawn_local(get_data(
            context.client.clone(),
            ctx.props().cid,
            ctx.link().callback(Msg::Data),
            ctx.link().callback(Msg::Identity),
        ));

        let mut follows = get_follow_list();

        if let Some((context, _)) = ctx.link().context::<ChannelContext>(Callback::noop()) {
            follows.insert(context.channel.get_address());
        }

        let (handle, regis) = AbortHandle::new_pair();

        spawn_local(stream_comments(
            context.client,
            follows,
            ctx.props().cid,
            ctx.link().callback(Msg::Reply),
            regis,
        ));

        Self {
            dt: String::new(),
            comment: None,
            identity: None,

            handle,
            reply: None,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Data(comment) => {
                self.dt = timestamp_to_datetime(comment.user_timestamp);
                self.comment = Some(comment);
            }
            Msg::Identity(id) => self.identity = Some(id),
            Msg::Reply(cid) => self.reply = Some(cid),
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if self.comment.is_none() || self.identity.is_none() {
            return html! {
                <Searching />
            };
        }

        let comment = self.comment.as_ref().unwrap();
        let identity = self.identity.as_ref().unwrap();

        let name = html! {
            <span class="icon-text">
                <span class="icon"><i class="fas fa-user"></i></span>
                <span> { &identity.display_name } </span>
            </span>
        };

        html! {
        <Media>
            <MediaLeft>
            {
                if let Some(ipld) = identity.avatar {
                    html! { <IPFSImage cid={ipld.link} size={ImageSize::Is64x64} rounded=true /> }
                } else {
                    html!()
                }
            }
            </MediaLeft>
            <MediaContent>
                <Level>
                    <LevelLeft>
                        <LevelItem>
                        {
                            if let Some(addr) = identity.channel_ipns {
                                html! {
                                    <Link<Route> to={Route::Channel{ addr: addr.into()}} >
                                        {name}
                                    </Link<Route>>
                                }
                            } else {
                                name
                            }
                        }
                        </LevelItem>
                        <LevelItem>
                            <span class="icon-text">
                                <span class="icon"><i class="fas fa-clock"></i></span>
                                <span> { &self.dt } </span>
                            </span>
                        </LevelItem>
                    </LevelLeft>
                </Level>
                <Content>
                    { &comment.text }
                </Content>
                {
                match self.reply {
                    Some(cid) => html!{ <Comment {cid} /> },
                    None => html!{},
                }
                }
            </MediaContent>
            <MediaRight>
                <Block>
                    <DagExplorer cid={ctx.props().cid} />
                </Block>
                <Level>
                    <LevelItem>
                        <CommentButton cid={ctx.props().cid} />
                    </LevelItem>
                    <LevelItem>
                        <ShareButton cid={ctx.props().cid} />
                    </LevelItem>
                </Level>
            </MediaRight>
        </Media>
            }
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        self.handle.abort();
    }
}

async fn get_data(
    ipfs: IpfsService,
    cid: Cid,
    data_cb: Callback<linked_data::comments::Comment>,
    id_cb: Callback<Identity>,
) {
    let (comment_res, id_res) = futures_util::join!(
        ipfs.dag_get::<&str, linked_data::comments::Comment>(cid, Some("/link")),
        ipfs.dag_get::<&str, Identity>(cid, Some("/link/identity"))
    );

    match comment_res {
        Ok(comment) => data_cb.emit(comment),
        Err(e) => error!(&format!("{:#?}", e)),
    };

    match id_res {
        Ok(id) => id_cb.emit(id),
        Err(e) => error!(&format!("{:#?}", e)),
    }
}

async fn stream_comments(
    ipfs: IpfsService,
    follows: HashSet<IPNSAddress>,
    content_cid: Cid,
    callback: Callback<Cid>,
    regis: AbortRegistration,
) {
    let defluencer = Defluencer::new(ipfs);

    let stream = defluencer
        .streaming_web_crawl(follows.into_iter())
        .try_filter_map(|(_, channel)| async move { Ok(channel.comment_index) })
        .map_ok(|index| defluencer.stream_content_comments(index, content_cid))
        .try_flatten();

    let mut stream = Abortable::new(stream, regis).boxed_local();

    while let Some(result) = stream.next().await {
        match result {
            Ok(cid) => callback.emit(cid),
            Err(e) => error!(&format!("{:#?}", e)),
        }
    }
}
