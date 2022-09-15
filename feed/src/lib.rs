#![cfg(target_arch = "wasm32")]

use std::collections::HashSet;

use cid::Cid;
use components::{navbar::NavigationBar, thumbnail::Thumbnail};

use defluencer::Defluencer;

use futures_util::{stream::FuturesUnordered, TryStreamExt};

use gloo_console::error;

use ipfs_api::IpfsService;

use linked_data::{channel::ChannelMetadata, media::Media, types::IPNSAddress};

use utils::ipfs::IPFSContext;

use yew::{platform::spawn_local, prelude::*};

/// social.defluencer.eth/#/feed/
///
/// The Personal Feed Page.
pub struct FeedPage {
    set: HashSet<Cid>,
    content: Vec<(Cid, Media)>,
}

pub enum Msg {
    Media((Cid, Media)),
}

impl Component for FeedPage {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (context, _) = ctx
            .link()
            .context::<IPFSContext>(Callback::noop())
            .expect("IPFS Context");

        let list = utils::follows::get_follow_list();

        spawn_local(stream_feed(
            context.client,
            list,
            ctx.link().callback(Msg::Media),
        ));

        Self {
            set: Default::default(),
            content: Default::default(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Media((cid, media)) => self.on_media(cid, media),
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
        <>
        <NavigationBar />
        {
            self.content.iter().rev().map(|&(cid, _)| {
                html! {
                    <Thumbnail key={cid.to_string()} {cid} />
                }
            }).collect::<Html>()
        }
        </>
        }
    }
}

impl FeedPage {
    fn on_media(&mut self, cid: Cid, media: Media) -> bool {
        if !self.set.insert(cid) {
            return false;
        }

        let index = self
            .content
            .binary_search_by(|(_, probe)| probe.user_timestamp().cmp(&media.user_timestamp()))
            .unwrap_or_else(|x| x);

        self.content.insert(index, (cid, media));

        true
    }
}

async fn stream_feed(
    ipfs: IpfsService,
    follows: HashSet<IPNSAddress>,
    callback: Callback<(Cid, Media)>,
) {
    use futures_util::StreamExt;

    let follow_count = follows.len();

    let stream: FuturesUnordered<_> = follows
        .into_iter()
        .map(|addr| ipfs.name_resolve(addr.into()))
        .collect();

    let defluencer = Defluencer::new(ipfs.clone());

    let mut stream = stream
        .map_ok(|cid| ipfs.dag_get::<&str, ChannelMetadata>(cid, None))
        .try_buffer_unordered(follow_count)
        .try_filter_map(|channel| async move { Ok(channel.content_index) })
        .map_ok(|index| defluencer.stream_content_rev_chrono(index).take(50))
        .try_flatten()
        .map(|result| {
            let ipfs = ipfs.clone();

            async move {
                match result {
                    Ok(cid) => match ipfs.dag_get::<&str, Media>(cid, Some("/link")).await {
                        Ok(media) => Ok((cid, media)),
                        Err(e) => Err(e.into()),
                    },
                    Err(e) => Err(e),
                }
            }
        })
        .buffer_unordered(100)
        .boxed_local();

    while let Some(result) = stream.next().await {
        match result {
            Ok(item) => callback.emit(item),
            Err(e) => {
                error!(&format!("{:#?}", e));
                continue;
            }
        }
    }
}
