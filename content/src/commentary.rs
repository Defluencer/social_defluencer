#![cfg(target_arch = "wasm32")]

use std::collections::HashSet;

use futures_util::{stream::FuturesUnordered, StreamExt, TryStreamExt};

use linked_data::{channel::ChannelMetadata, types::IPNSAddress};

use utils::{defluencer::ChannelContext, follows::get_follow_list, ipfs::IPFSContext};

use ybc::{Container, Section};

use yew::{platform::spawn_local, prelude::*};

use cid::Cid;

use gloo_console::{error, info};

use components::comment::Comment;

use ipfs_api::IpfsService;

use defluencer::Defluencer;

//const MAX_CRAWL_RESULT: usize = 50;

#[derive(Properties, PartialEq)]
pub struct Props {
    /// signed link to Media Content Cid
    pub cid: Cid,
}

pub struct Commentary {
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

        spawn_local(stream_comments(
            context.client,
            ctx.props().cid,
            follows,
            comment_cb.clone(),
        ));

        Self {
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
}

async fn stream_comments(
    ipfs: IpfsService,
    content_cid: Cid,
    follows: HashSet<IPNSAddress>,
    callback: Callback<Cid>,
) {
    let defluencer = Defluencer::new(ipfs.clone());

    //TODO fix web crawl

    let follow_count = follows.len();

    let stream: FuturesUnordered<_> = follows
        .into_iter()
        .map(|addr| ipfs.name_resolve(addr.into()))
        .collect();

    let mut stream = stream
        .map_ok(|cid| ipfs.dag_get::<&str, ChannelMetadata>(cid, None))
        .try_buffer_unordered(follow_count)
        .try_filter_map(|channel| async move { Ok(channel.comment_index) })
        .map_ok(|index| defluencer.stream_comments(index, content_cid))
        .try_flatten()
        .boxed_local();

    while let Some(result) = stream.next().await {
        match result {
            Ok(cid) => callback.emit(cid),
            Err(e) => error!(&format!("{:#?}", e)),
        }
    }
}
