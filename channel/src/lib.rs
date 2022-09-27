#![cfg(target_arch = "wasm32")]

mod manage_content;

use linked_data::identity::Identity;

use manage_content::ManageContent;

use std::collections::{HashMap, HashSet, VecDeque};

use components::pure::{DagExplorer, IPFSImage, Thumbnail};

use components::pure::{NavigationBar, Searching};

use futures_util::stream::AbortHandle;

#[cfg(debug_assertions)]
use gloo_console::info;

use linked_data::{channel::ChannelMetadata, media::Media};

use utils::{
    defluencer::{ChannelContext, UserContext},
    ipfs::IPFSContext,
};

use ybc::{Box, Button, Container, ImageSize, Level, LevelItem, LevelLeft, LevelRight, Section};

use yew::{platform::spawn_local, prelude::*};

use cid::Cid;

#[derive(Properties, PartialEq)]
pub struct Props {
    /// Channel Address
    pub addr: Cid,
}

//TODO find a way to tell if live, then display video

/// social.defluencer.eth/#/channel/<IPNS_HERE>
///
/// A specific channel page
pub struct ChannelPage {
    sub_handle: AbortHandle,
    stream_handle: Option<AbortHandle>,

    metadata: Option<ChannelMetadata>,

    content: VecDeque<(Cid, Media)>,
    content_cb: Callback<(Cid, Media)>,

    identity_cb: Callback<(Cid, Identity)>,
    identities: HashMap<Cid, Identity>,

    own_channel: bool,

    follow_cb: Callback<MouseEvent>,
    following: bool,
}

pub enum Msg {
    Update(ChannelMetadata),
    Content((Cid, Media)),
    Identity((Cid, Identity)),
    Follow,
}

impl Component for ChannelPage {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        #[cfg(debug_assertions)]
        info!("Channel Page Create");

        let (context, _) = ctx
            .link()
            .context::<IPFSContext>(Callback::noop())
            .expect("IPFS Context");

        spawn_local(utils::r#async::get_channels(
            context.client.clone(),
            ctx.link().callback(Msg::Update),
            HashSet::from([ctx.props().addr.into()]),
        ));

        let (sub_handle, regis) = AbortHandle::new_pair();

        spawn_local(utils::r#async::channel_subscribe(
            context.client.clone(),
            ctx.link().callback(Msg::Update),
            ctx.props().addr.into(),
            regis,
        ));

        let content_cb = ctx.link().callback(Msg::Content);
        let identity_cb = ctx.link().callback(Msg::Identity);

        let mut channel = false;

        if ctx
            .link()
            .context::<UserContext>(Callback::noop())
            .is_some()
        {
            if let Some((context, _)) = ctx.link().context::<ChannelContext>(Callback::noop()) {
                if context.channel.get_address() == ctx.props().addr.into() {
                    channel = true;
                }
            }
        }

        let follow_cb = ctx.link().callback(|_| Msg::Follow);

        let following = {
            let list = utils::follows::get_follow_list();

            list.contains(&ctx.props().addr.into())
        };

        Self {
            sub_handle,
            stream_handle: None,

            metadata: None,

            content: Default::default(),
            content_cb,

            identities: Default::default(),
            identity_cb,

            own_channel: channel,

            follow_cb,
            following,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        #[cfg(debug_assertions)]
        info!("Channel Page Update");

        match msg {
            Msg::Update(metadata) => self.on_channel_update(ctx, metadata),
            Msg::Content((cid, media)) => self.on_content(ctx, cid, media),
            Msg::Identity((cid, identity)) => self.identities.insert(cid, identity).is_none(),
            Msg::Follow => self.on_follow(ctx),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        #[cfg(debug_assertions)]
        info!("Channel Page View");

        match &self.metadata {
            Some(meta) => self.render_channel(ctx, meta),
            None => self.render_no_channel(),
        }
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        #[cfg(debug_assertions)]
        info!("Channel Page Destroy");

        self.sub_handle.abort();

        if let Some(handle) = self.stream_handle.take() {
            handle.abort();
        }
    }
}

impl ChannelPage {
    fn render_channel(&self, ctx: &Context<ChannelPage>, meta: &ChannelMetadata) -> Html {
        html! {
        <>
            <NavigationBar />
            <Section>
                <Container>
                if let Some(identity) = self.identities.get(&meta.identity.link) {
                    //TODO display channel branding and general info
                    <Level>
                        <LevelLeft>
                            if let Some(avatar) = identity.avatar {
                                <LevelItem>
                                    <IPFSImage cid={avatar.link} size={ImageSize::Is64x64} rounded=true />
                                </LevelItem>
                            }
                            <LevelItem>
                                <span class="icon-text">
                                    <span class="icon"><i class="fas fa-user"></i></span>
                                    <span> { &identity.display_name } </span>
                                </span>
                            </LevelItem>
                            <LevelItem>
                                <Button onclick={self.follow_cb.clone()} >
                                {
                                    if self.following {"Unfollow"} else {"Follow"}
                                }
                                </Button>
                            </LevelItem>
                        </LevelLeft>
                        <LevelRight>
                            <DagExplorer key={meta.identity.link.to_string()} cid={meta.identity.link} />
                        </LevelRight>
                    </Level>
                    if self.own_channel
                    {
                        <ManageContent addr={ctx.props().addr} />
                    }
                }
                </Container>
            </Section>
            <Section>
                <Container>
                {
                    self.content
                        .iter()
                        .filter_map(|(cid, media)| {
                            let cid = *cid;
                            let media = media.clone();

                            let identity = match self.identities.get(&media.identity().link) {
                                Some(id) => id.clone(),
                                None => return None,
                            };

                            return Some(html! {
                            <Box>
                                <Thumbnail key={cid.to_string()} {cid} {media} {identity} />
                            </Box>
                            });
                        })
                        .collect::<Html>()
                }
                </Container>
            </Section>
        </>
        }
    }

    fn render_no_channel(&self) -> Html {
        html! {
        <>
            <NavigationBar />
            <Section>
                <Container>
                    <Searching />
                </Container>
            </Section>
        </>
        }
    }

    fn on_channel_update(&mut self, ctx: &Context<Self>, metadata: ChannelMetadata) -> bool {
        if self.metadata.as_ref() == Some(&metadata) {
            return false;
        }

        let (context, _) = ctx
            .link()
            .context::<IPFSContext>(Callback::noop())
            .expect("IPFS Context");
        let ipfs = context.client;

        if let Some(index) = metadata.content_index {
            if let Some(handle) = self.stream_handle.take() {
                handle.abort();
            }

            self.content.clear();

            let (stream_handle, regis) = AbortHandle::new_pair();

            spawn_local(utils::r#async::stream_content(
                ipfs.clone(),
                self.content_cb.clone(),
                index,
                regis,
            ));

            self.stream_handle = Some(stream_handle);
        }

        if !self.identities.contains_key(&metadata.identity.link) {
            spawn_local(utils::r#async::get_identity(
                ipfs,
                metadata.identity.link,
                self.identity_cb.clone(),
            ));
        }

        self.metadata = Some(metadata);

        true
    }

    fn on_content(&mut self, ctx: &Context<Self>, cid: Cid, media: Media) -> bool {
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

        self.content.push_back((cid, media));

        true
    }

    fn on_follow(&mut self, ctx: &Context<Self>) -> bool {
        let mut list = utils::follows::get_follow_list();

        if !self.following {
            list.insert(ctx.props().addr.into());
        } else {
            list.remove(&ctx.props().addr.into());
        }

        utils::follows::set_follow_list(list);

        self.following = !self.following;

        true
    }
}
