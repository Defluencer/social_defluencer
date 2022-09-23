#![cfg(target_arch = "wasm32")]

use std::collections::HashSet;

use cid::Cid;

use components::{navbar::NavigationBar, thumbnail::Thumbnail};

use defluencer::Defluencer;

use futures_util::{
    stream::{AbortHandle, AbortRegistration, FuturesUnordered, SelectAll},
    StreamExt, TryStreamExt,
};

use gloo_console::{error, info};

use ipfs_api::IpfsService;

use linked_data::{channel::ChannelMetadata, media::Media, types::IPNSAddress};

use utils::ipfs::IPFSContext;

use ybc::{Container, Section};

use yew::{platform::spawn_local, prelude::*};

/// social.defluencer.eth/#/feed/
///
/// The Personal Feed Page.
pub struct FeedPage {
    handle: AbortHandle,

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
        #[cfg(debug_assertions)]
        info!("Feed Page Create");

        let (context, _) = ctx
            .link()
            .context::<IPFSContext>(Callback::noop())
            .expect("IPFS Context");

        let list = utils::follows::get_follow_list();

        let (handle, regis) = AbortHandle::new_pair();

        spawn_local(aggregate(
            context.client,
            list.into_iter(),
            0,
            50,
            ctx.link().callback(Msg::Media),
            regis,
        ));

        Self {
            handle,
            set: Default::default(),
            content: Default::default(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        #[cfg(debug_assertions)]
        info!("Feed Page Update");

        match msg {
            Msg::Media((cid, media)) => self.on_media(cid, media),
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        #[cfg(debug_assertions)]
        info!("Feed Page View");

        html! {
        <>
        <NavigationBar />
        <Section>
            <Container>
            {
                self.content.iter().rev().map(|&(cid, _)| {
                    html! {
                        <Thumbnail key={cid.to_string()} {cid} />
                    }
                }).collect::<Html>()
            }
            </Container>
        </Section>
        </>
        }
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        #[cfg(debug_assertions)]
        info!("Feed Page Destroy");

        self.handle.abort();
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

async fn aggregate(
    ipfs: IpfsService,
    addresses: impl Iterator<Item = IPNSAddress>,
    page_index: usize,
    pagination_length: usize,
    content_cb: Callback<(Cid, Media)>,
    _regis: AbortRegistration,
) {
    let defluencer = Defluencer::new(ipfs.clone());

    let mut update_pool: FuturesUnordered<_> = addresses
        .into_iter()
        .map(|addr| ipfs.name_resolve(addr.into()))
        .collect();

    let mut metadata_set = HashSet::new();
    let mut metadata_pool = FuturesUnordered::<_>::new();

    let mut content_pool = SelectAll::<_>::new();

    let mut content_set = HashSet::new();
    let mut media_pool = FuturesUnordered::<_>::new();

    loop {
        //TODO find a way to terminate this gracefully OR make one stream without sets

        futures_util::select! {
            result = update_pool.try_next() => {
                let cid = match result {
                    Ok(option) => match option {
                        Some(cid) => cid,
                        None => continue,
                    },
                    Err(e) => {
                        error!(&format!("{:#?}", e));
                        continue;
                    },
                };

                if !metadata_set.insert(cid) {
                    continue;
                }

                metadata_pool.push(ipfs.dag_get::<&str, ChannelMetadata>(cid, None));
            },
            result = metadata_pool.try_next() => {
                let metadata = match result {
                    Ok(option) => match option {
                        Some(dag) => dag,
                        None => continue,
                    },
                    Err(e) => {
                        error!(&format!("{:#?}", e));
                        continue;
                    },
                };

                if let Some(index) = metadata.content_index {
                    content_pool.push(
                    defluencer
                    .stream_content_rev_chrono(index)
                    .skip(page_index * pagination_length)
                    .take(pagination_length)
                    .boxed_local()
                    );
                }
            },
            result = content_pool.try_next() => {
                let cid = match result {
                    Ok(option) => match option {
                        Some(cid) => cid,
                        None => continue,
                    },
                    Err(e) => {
                        error!(&format!("{:#?}", e));
                        continue;
                    },
                };

                if !content_set.insert(cid) {
                    continue;
                }

                media_pool.push({
                    let ipfs = ipfs.clone();

                    async move {
                        (cid, ipfs.dag_get::<&str, Media>(cid, Some("/link")).await)
                    }
                });
            },
            option = media_pool.next() => {
                let (cid, media) = match option {
                    Some((cid, result)) => match result {
                        Ok(dag) => (cid, dag),
                        Err(e) => {
                            error!(&format!("{:#?}", e));
                            continue;
                        }
                    },
                    None => continue,
                };

                content_cb.emit((cid, media));
            },
            complete => return,
        }
    }
}
