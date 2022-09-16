#![cfg(target_arch = "wasm32")]

use std::collections::HashSet;

use futures_util::{
    stream::{AbortHandle, AbortRegistration, Abortable},
    StreamExt, TryStreamExt,
};

use linked_data::types::IPNSAddress;

use utils::{defluencer::ChannelContext, follows::get_follow_list, ipfs::IPFSContext};

use ybc::{Container, Section};

use yew::{platform::spawn_local, prelude::*};

use cid::Cid;

use gloo_console::{error, info};

use components::comment::Comment;

use ipfs_api::IpfsService;

use defluencer::Defluencer;

#[derive(Properties, PartialEq)]
pub struct Props {
    /// signed link to Media Content Cid
    pub cid: Cid,
}

pub struct Commentary {
    handle: AbortHandle,

    comments_set: HashSet<Cid>,
}

pub enum Msg {
    Comment(Cid),
}

impl Component for Commentary {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        #[cfg(debug_assertions)]
        info!("Commentary Create");

        let (context, _) = ctx
            .link()
            .context::<IPFSContext>(Callback::noop())
            .expect("IPFS Context");

        let mut follows = get_follow_list();

        if let Some((context, _)) = ctx.link().context::<ChannelContext>(Callback::noop()) {
            follows.insert(context.channel.get_address());
        }

        let comment_cb = ctx.link().callback(Msg::Comment);

        let (handle, regis) = AbortHandle::new_pair();

        spawn_local(stream_comments(
            context.client,
            follows,
            ctx.props().cid,
            comment_cb.clone(),
            regis,
        ));

        Self {
            handle,
            comments_set: HashSet::default(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        #[cfg(debug_assertions)]
        info!("Commentary Update");

        match msg {
            Msg::Comment(cid) => self.comments_set.insert(cid),
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        #[cfg(debug_assertions)]
        info!("Commentary View");

        let comment_list = self
            .comments_set
            .iter()
            .map(|cid| {
                let cid = *cid;

                html! {
                    <Comment key={cid.to_string()} {cid} />
                }
            })
            .collect::<Html>();

        html! {
        <Section>
            <Container>
                { comment_list }
            </Container>
        </Section>
        }
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        #[cfg(debug_assertions)]
        info!("Commentary Destroy");

        self.handle.abort();
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
