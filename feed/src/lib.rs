#![cfg(target_arch = "wasm32")]

use std::collections::HashMap;

use cid::Cid;

use components::pure::{NavigationBar, Thumbnail};

use futures_util::stream::AbortHandle;

#[cfg(debug_assertions)]
use gloo_console::info;

use linked_data::{channel::ChannelMetadata, identity::Identity, media::Media};

use utils::ipfs::IPFSContext;

use ybc::{Container, Section};

use yew::{platform::spawn_local, prelude::*};

/// social.defluencer.eth/#/feed/
///
/// The Personal Feed Page display all followed channel content
pub struct FeedPage {
    //channels: Vec<ChannelMetadata>,
    handles: Vec<AbortHandle>,

    content_cb: Callback<(Cid, Media)>,
    content: HashMap<Cid, Media>,

    /// Media Cid sorted by timestamps.
    content_order: Vec<Cid>,

    identity_cb: Callback<(Cid, Identity)>,
    identities: HashMap<Cid, Identity>,
}

pub enum Msg {
    Channel(ChannelMetadata),
    Content((Cid, Media)),
    Identity((Cid, Identity)),
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

        let set = utils::follows::get_follow_list();

        spawn_local(utils::r#async::get_channels(
            context.client.clone(),
            ctx.link().callback(Msg::Channel),
            set,
        ));

        let content_cb = ctx.link().callback(Msg::Content);
        let identity_cb = ctx.link().callback(Msg::Identity);

        Self {
            handles: Default::default(),

            content_cb,
            content: Default::default(),
            content_order: Default::default(),

            identity_cb,
            identities: Default::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        #[cfg(debug_assertions)]
        info!("Feed Page Update");

        match msg {
            Msg::Channel(meta) => self.on_channel(ctx, meta),
            Msg::Content((cid, media)) => self.on_content(ctx, cid, media),
            Msg::Identity((cid, identity)) => self.identities.insert(cid, identity).is_none(),
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
                self.content_order.iter().rev().filter_map(|&cid| {
                    let media = self.content.get(&cid).unwrap().clone();

                    let identity = match self.identities.get(&media.identity().link) {
                        Some(id) => id.clone(),
                        None => return None,
                    };

                    Some(html! {
                        <Thumbnail key={cid.to_string()} {cid} {media} {identity} />
                    })
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

        for handle in &self.handles {
            handle.abort();
        }
    }
}

impl FeedPage {
    fn on_channel(&mut self, ctx: &Context<Self>, metadata: ChannelMetadata) -> bool {
        let (context, _) = ctx
            .link()
            .context::<IPFSContext>(Callback::noop())
            .expect("IPFS Context");
        let ipfs = context.client;

        if let Some(index) = metadata.content_index {
            let (handle, regis) = AbortHandle::new_pair();

            spawn_local(utils::r#async::stream_content(
                ipfs.clone(),
                self.content_cb.clone(),
                index,
                regis,
            ));

            self.handles.push(handle);
        }

        if !self.identities.contains_key(&metadata.identity.link) {
            spawn_local(utils::r#async::get_identity(
                ipfs,
                metadata.identity.link,
                self.identity_cb.clone(),
            ));
        }

        false
    }

    fn on_content(&mut self, ctx: &Context<Self>, cid: Cid, media: Media) -> bool {
        if self.content.contains_key(&cid) {
            return false;
        }

        if !self.identities.contains_key(&media.identity().link) {
            let (context, _) = ctx
                .link()
                .context::<IPFSContext>(Callback::noop())
                .expect("IPFS Context");

            spawn_local(utils::r#async::get_identity(
                context.client,
                media.identity().link,
                self.identity_cb.clone(),
            ));
        }

        let index = self
            .content_order
            .binary_search_by(|cid| {
                self.content
                    .get(&cid)
                    .unwrap()
                    .user_timestamp()
                    .cmp(&media.user_timestamp())
            })
            .unwrap_or_else(|x| x);

        self.content_order.insert(index, cid);
        self.content.insert(cid, media);

        true
    }
}

/* async fn aggregate(
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
} */
