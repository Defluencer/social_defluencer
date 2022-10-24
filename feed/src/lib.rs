#![cfg(target_arch = "wasm32")]

use std::collections::HashMap;

use cid::Cid;

use components::pure::{NavigationBar, Searching, Thumbnail};

use futures_util::stream::AbortHandle;

#[cfg(debug_assertions)]
use gloo_console::info;

use linked_data::{channel::ChannelMetadata, identity::Identity, media::Media, types::IPNSAddress};

use utils::ipfs::IPFSContext;

use ybc::{Container, HeaderSize, Section, Title};

use yew::{platform::spawn_local, prelude::*};

/// social.defluencer.eth/#/feed/
///
/// The Personal Feed Page display all subcribed channel content
pub struct FeedPage {
    latest_roots: HashMap<IPNSAddress, Cid>,

    sub_handles: HashMap<IPNSAddress, AbortHandle>,
    stream_handles: HashMap<IPNSAddress, AbortHandle>,

    content_cb: Callback<(Cid, Media)>,
    content: HashMap<Cid, Media>,

    /// Media Cid sorted by timestamps.
    content_order: Vec<Cid>,

    identity_cb: Callback<(Cid, Identity)>,
    identities: HashMap<Cid, Identity>,
}

pub enum Msg {
    Channel((IPNSAddress, Cid, ChannelMetadata)),
    Content((Cid, Media)),
    Identity((Cid, Identity)),
}

impl Component for FeedPage {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        #[cfg(debug_assertions)]
        info!("Feed Page Create");

        let set = utils::subscriptions::get_sub_list();

        let channel_cb = ctx.link().callback(Msg::Channel);

        let content_cb = ctx.link().callback(Msg::Content);
        let identity_cb = ctx.link().callback(Msg::Identity);

        let mut sub_handles = HashMap::with_capacity(set.len());

        if let Some((context, _)) = ctx.link().context::<IPFSContext>(Callback::noop()) {
            let ipfs = context.client;

            spawn_local(utils::r#async::get_channels(
                ipfs.clone(),
                channel_cb.clone(),
                set.clone(),
            ));

            for addr in set {
                let (handle, regis) = AbortHandle::new_pair();

                spawn_local(utils::r#async::channel_subscribe(
                    ipfs.clone(),
                    channel_cb.clone(),
                    addr,
                    regis,
                ));

                sub_handles.insert(addr, handle);
            }
        }

        Self {
            latest_roots: Default::default(),

            sub_handles,
            stream_handles: Default::default(),

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
            Msg::Channel((addr, cid, meta)) => self.on_channel_update(ctx, addr, cid, meta),
            Msg::Content((cid, media)) => self.on_content_discovered(ctx, cid, media),
            Msg::Identity((cid, identity)) => self.identities.insert(cid, identity).is_none(),
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        #[cfg(debug_assertions)]
        info!("Feed Page View");

        if self.sub_handles.is_empty() {
            return html! {
            <>
            <NavigationBar />
            <Section>
                <Container>
                    <Title classes={classes!("has-text-centered")} size={HeaderSize::Is5} >
                        {"Subscribe to your favorite channel to build your content feed."}
                    </Title>
                </Container>
            </Section>
            </>
            };
        }

        if self.content_order.is_empty() {
            return html! {
            <>
            <NavigationBar />
            <Section>
                <Container>
                    <Searching />
                </Container>
            </Section>
            </>
            };
        }

        html! {
        <>
        <NavigationBar />
        <Section>
            <Container>
            {
                self.content_order.iter().rev().filter_map(|&cid| {
                    let media = match self.content.get(&cid) {
                        Some(media) => media.clone(),
                        None => return None,
                    };

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

        for (sub, stream) in self.sub_handles.values().zip(self.stream_handles.values()) {
            sub.abort();
            stream.abort();
        }
    }
}

impl FeedPage {
    fn on_channel_update(
        &mut self,
        ctx: &Context<Self>,
        addr: IPNSAddress,
        cid: Cid,
        metadata: ChannelMetadata,
    ) -> bool {
        let ipfs = match ctx.link().context::<IPFSContext>(Callback::noop()) {
            Some((context, _)) => context.client,
            None => return false,
        };

        if self.latest_roots.get(&addr) == Some(&cid) {
            return false;
        }

        if let Some(index) = metadata.content_index {
            let (handle, regis) = AbortHandle::new_pair();

            spawn_local(utils::r#async::stream_content(
                ipfs.clone(),
                self.content_cb.clone(),
                index,
                regis,
            ));

            if let Some(handle) = self.stream_handles.insert(addr, handle) {
                handle.abort();
            }

            if !self.identities.contains_key(&metadata.identity.link) {
                spawn_local(utils::r#async::dag_get(
                    ipfs,
                    metadata.identity.link,
                    self.identity_cb.clone(),
                ));
            }
        }

        self.latest_roots.insert(addr, cid);

        false
    }

    fn on_content_discovered(&mut self, ctx: &Context<Self>, cid: Cid, media: Media) -> bool {
        let ipfs = match ctx.link().context::<IPFSContext>(Callback::noop()) {
            Some((context, _)) => context.client,
            None => return false,
        };

        if self.content.contains_key(&cid) {
            return false;
        }

        if !self.identities.contains_key(&media.identity().link) {
            spawn_local(utils::r#async::dag_get(
                ipfs,
                media.identity().link,
                self.identity_cb.clone(),
            ));
        }

        let index = self
            .content_order
            .binary_search_by(|cid| {
                self.content[cid]
                    .user_timestamp()
                    .cmp(&media.user_timestamp())
            })
            .unwrap_or_else(|x| x);

        self.content_order.insert(index, cid);
        self.content.insert(cid, media);

        true
    }
}
