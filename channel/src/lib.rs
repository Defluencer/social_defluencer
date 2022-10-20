#![cfg(target_arch = "wasm32")]

mod manage_content;

use defluencer::Defluencer;

use ipfs_api::IpfsService;

use manage_content::ManageContent;

use linked_data::{identity::Identity, types::IPNSAddress};

use std::collections::{HashMap, HashSet, VecDeque};

use components::{pure::{DagExplorer, Followee, IPFSImage, NavigationBar, Searching, Thumbnail}, Route};

use futures_util::stream::AbortHandle;

#[cfg(debug_assertions)]
use gloo_console::info;

use linked_data::{channel::ChannelMetadata, media::Media};

use utils::{
    defluencer::{ChannelContext, UserContext},
    ipfs::IPFSContext,
};

use ybc::{
    Alignment, Block, Button, ButtonRouter, Container, Content, ImageSize, Level, LevelItem,
    LevelLeft, LevelRight, MediaContent, MediaLeft, MediaRight, Section, Size, Tabs,
};

use yew::{platform::spawn_local, prelude::*};

use cid::Cid;

#[derive(Properties, PartialEq)]
pub struct Props {
    /// Channel Address
    pub addr: IPNSAddress,
}

//TODO find a way to tell if live, then display video

/// social.defluencer.eth/#/channel/<IPNS_HERE>
///
/// A specific channel page
pub struct ChannelPage {
    addr: IPNSAddress,

    sub_handle: AbortHandle,
    stream_handle: Option<AbortHandle>,

    metadata: Option<ChannelMetadata>,
    update_cb: Callback<(IPNSAddress, Cid, ChannelMetadata)>,

    content: VecDeque<(Cid, Media)>,
    content_cb: Callback<(Cid, Media)>,

    identity_cb: Callback<(Cid, Identity)>,
    identities: HashMap<Cid, Identity>,

    own_channel: bool,

    subscribe_cb: Callback<MouseEvent>,
    subscription: bool,

    filter: Filter,

    followees: HashMap<Cid, Identity>,
    followees_cb: Callback<HashMap<Cid, Identity>>,
}

#[derive(PartialEq, Debug)]
pub enum Filter {
    None,
    Articles,
    Videos,
    Comments,
    Followees,
}

pub enum Msg {
    Update((IPNSAddress, Cid, ChannelMetadata)),
    Content((Cid, Media)),
    Identity((Cid, Identity)),
    Follow,
    Filter(Filter),
    Followees(HashMap<Cid, Identity>),
}

impl Component for ChannelPage {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        #[cfg(debug_assertions)]
        info!("Channel Page Create");

        let update_cb = ctx.link().callback(Msg::Update);
        let content_cb = ctx.link().callback(Msg::Content);
        let identity_cb = ctx.link().callback(Msg::Identity);
        let follow_cb = ctx.link().callback(|_| Msg::Follow);
        let followees_cb = ctx.link().callback(Msg::Followees);

        let addr = ctx.props().addr;

        let (sub_handle, regis) = AbortHandle::new_pair();

        if let Some((context, _)) = ctx.link().context::<IPFSContext>(Callback::noop()) {
            spawn_local(utils::r#async::get_channels(
                context.client.clone(),
                update_cb.clone(),
                HashSet::from([addr]),
            ));

            spawn_local(utils::r#async::channel_subscribe(
                context.client.clone(),
                update_cb.clone(),
                addr,
                regis,
            ));
        };

        let mut own_channel = false;

        if ctx
            .link()
            .context::<UserContext>(Callback::noop())
            .is_some()
        {
            if let Some((context, _)) = ctx.link().context::<ChannelContext>(Callback::noop()) {
                if context.channel.get_address() == addr {
                    own_channel = true;
                }
            }
        }

        let following = {
            let list = utils::follows::get_follow_list();

            list.contains(&addr)
        };

        Self {
            addr,

            sub_handle,
            stream_handle: None,

            metadata: None,
            update_cb,

            content: Default::default(),
            content_cb,

            identities: Default::default(),
            identity_cb,

            own_channel,

            subscribe_cb: follow_cb,
            subscription: following,

            filter: Filter::None,

            followees: Default::default(),
            followees_cb,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        #[cfg(debug_assertions)]
        info!("Channel Page Update");

        match msg {
            Msg::Update((_, _, metadata)) => self.on_channel_update(ctx, metadata),
            Msg::Content((cid, media)) => self.on_content_discovered(ctx, cid, media),
            Msg::Identity((cid, identity)) => self.identities.insert(cid, identity).is_none(),
            Msg::Follow => self.on_follow(ctx),
            Msg::Filter(filter) => self.on_filtering(filter),
            Msg::Followees(followees) => self.on_followees(followees),
        }
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        #[cfg(debug_assertions)]
        info!("Channel Page Changed");

        let ipfs = match ctx.link().context::<IPFSContext>(Callback::noop()) {
            Some((context, _)) => context.client,
            None => return false,
        };

        if self.addr != ctx.props().addr {
            self.addr = ctx.props().addr;
            self.metadata.take();
            self.content.clear();
            self.sub_handle.abort();
            self.own_channel = false;
            self.filter = Filter::None;
            self.followees.clear();

            if let Some(handle) = self.stream_handle.take() {
                handle.abort();
            }

            let (sub_handle, regis) = AbortHandle::new_pair();
            self.sub_handle = sub_handle;

            spawn_local(utils::r#async::get_channels(
                ipfs.clone(),
                self.update_cb.clone(),
                HashSet::from([self.addr]),
            ));

            spawn_local(utils::r#async::channel_subscribe(
                ipfs.clone(),
                self.update_cb.clone(),
                self.addr,
                regis,
            ));

            if let Some((context, _)) = ctx.link().context::<ChannelContext>(Callback::noop()) {
                if context.channel.get_address() == self.addr {
                    self.own_channel = true;
                }
            }

            self.subscription = utils::follows::get_follow_list().contains(&self.addr);

            return true;
        }

        false
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
    fn render_channel(&self, ctx: &Context<Self>, meta: &ChannelMetadata) -> Html {
        html! {
        <>
        <NavigationBar />
        <Container>
        if let Some(identity) = self.identities.get(&meta.identity.link) {
            if let Some(banner) = identity.banner {
                <Block>
                    <IPFSImage cid={banner.link} size={ImageSize::Is3by1} rounded=false />
                </Block>
            }
            <Block>
                <ybc::Media>
                    <MediaLeft>
                    if let Some(avatar) = identity.avatar {
                        <IPFSImage cid={avatar.link} size={ImageSize::Is64x64} rounded=true />
                    }
                    </MediaLeft>
                    <MediaContent>
                        <Level>
                        <LevelLeft>
                            <LevelItem>
                                <span class="icon-text">
                                    <span class="icon"><i class="fas fa-user"></i></span>
                                    <span><strong>{&identity.name}</strong></span>
                                </span>
                            </LevelItem>
                            <LevelItem>
                                <Button classes={classes!("is-small", "is-rounded")} onclick={self.subscribe_cb.clone()} >
                                {
                                    if self.subscription {"Unsubscribe"} else {"Subscribe"}
                                }
                                </Button>
                            </LevelItem>
                            if let Some(live) = meta.live {
                            <LevelItem>
                                <ButtonRouter<Route> classes={classes!("is-small", "is-rounded")} route={Route::Live{ cid: live.link }} >
                                    {"Live"}
                                </ButtonRouter<Route>>
                            </LevelItem>
                            }
                        </LevelLeft>
                        <LevelRight>
                        if let Some(addr) = identity.ipns_addr {
                            <LevelItem>
                                <span class="icon-text">
                                    <span class="icon"><i class="fa-solid fa-fingerprint"></i></span>
                                    <span><small>{addr.to_string()}</small></span>
                                </span>
                            </LevelItem>
                        }
                        </LevelRight>
                        </Level>
                    if let Some(eth_addr) = &identity.eth_addr {
                        <span class="icon-text">
                            <span class="icon"><i class="fa-brands fa-ethereum"></i></span>
                            <span><small>{eth_addr}</small></span>
                        </span>
                    }
                    <br/>
                    if let Some(btc_addr) = &identity.btc_addr {
                        <span class="icon-text">
                            <span class="icon"><i class="fa-brands fa-btc"></i></span>
                            <span><small>{btc_addr}</small></span>
                        </span>
                    }
                    <br/>
                    if let Some(bio) = &identity.bio {
                        <Content>{ bio }</Content>
                    }
                    </MediaContent>
                    <MediaRight>
                        <DagExplorer key={meta.identity.link.to_string()} cid={meta.identity.link} />
                    </MediaRight>
                </ybc::Media>
            </Block>
            if self.own_channel
            {
                <ManageContent addr={ctx.props().addr} />
            }
        }
            <Block>
                <Tabs alignment={Alignment::Centered} size={Size::Normal} boxed=true toggle=true >
                    <li class={ if self.filter == Filter::Articles {"is-active"} else {""} } >
                        <Button onclick={ctx.link().callback(|_| Msg::Filter(Filter::Articles))} >
                            <span class="icon-text">
                                <span class="icon"><i class="fa-solid fa-newspaper"></i></span>
                                <span>{"Articles"}</span>
                            </span>
                        </Button>
                    </li>
                    <li class={ if self.filter == Filter::Videos {"is-active"} else {""} } >
                        <Button onclick={ctx.link().callback(|_| Msg::Filter(Filter::Videos))} >
                            <span class="icon-text">
                                <span class="icon"><i class="fas fa-video"></i></span>
                                <span>{"Videos"}</span>
                            </span>
                        </Button>
                    </li>
                    <li class={ if self.filter == Filter::Comments {"is-active"} else {""} } >
                        <Button onclick={ctx.link().callback(|_| Msg::Filter(Filter::Comments))} >
                            <span class="icon-text">
                                <span class="icon"><i class="fa-solid fa-comment"></i></span>
                                <span>{"Comments"}</span>
                            </span>
                        </Button>
                    </li>
                    <li class={ if self.filter == Filter::Followees {"is-active"} else {""} } >
                        <Button onclick={ctx.link().callback(|_| Msg::Filter(Filter::Followees))} >
                            <span class="icon-text">
                                <span class="icon"><i class="fas fa-user"></i></span>
                                <span>{"Followees"}</span>
                            </span>
                        </Button>
                    </li>
                    <li class={ if self.filter == Filter::None {"is-active"} else {""} } >
                        <Button onclick={ctx.link().callback(|_| Msg::Filter(Filter::None))} >
                            {"No Filter"}
                        </Button>
                    </li>
                </Tabs>
            </Block>
            { self.render_content() }
        </Container>
        </>
        }
    }

    fn render_content(&self) -> Html {
        if self.filter == Filter::Followees {
            return self
                .followees
                .iter()
                .map(|(cid, identity)| {
                    let cid = *cid;
                    let identity = identity.clone();

                    html! {
                    <Block>
                        <Followee {cid} {identity} />
                    </Block>
                    }
                })
                .collect::<Html>();
        }

        if self.content.is_empty() {
            return html! {<Searching />};
        }

        self.content
            .iter()
            .filter_map(|(cid, media)| {
                if self.filter != Filter::None {
                    match media {
                        Media::Blog(_) => {
                            if self.filter != Filter::Articles {
                                return None;
                            }
                        }
                        Media::Video(_) => {
                            if self.filter != Filter::Videos {
                                return None;
                            }
                        }
                        Media::Comment(_) => {
                            if self.filter != Filter::Comments {
                                return None;
                            }
                        }
                    }
                }

                let cid = *cid;
                let media = media.clone();

                let identity = match self.identities.get(&media.identity().link) {
                    Some(id) => id.clone(),
                    None => return None,
                };

                return Some(html! {
                <Block>
                    <Thumbnail key={cid.to_string()} {cid} {media} {identity} />
                </Block>
                });
            })
            .collect::<Html>()
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
        let ipfs = match ctx.link().context::<IPFSContext>(Callback::noop()) {
            Some((context, _)) => context.client,
            None => return false,
        };

        if self.metadata.as_ref() == Some(&metadata) {
            return false;
        }

        if metadata.follows.is_some() {
            spawn_local(get_followees(
                ipfs.clone(),
                metadata.clone(),
                self.followees_cb.clone(),
            ));
        }

        if !self.identities.contains_key(&metadata.identity.link) {
            spawn_local(utils::r#async::dag_get(
                ipfs.clone(),
                metadata.identity.link,
                self.identity_cb.clone(),
            ));
        }

        if let Some(idx) = metadata.content_index {
            self.content.clear();

            let (handle, regis) = AbortHandle::new_pair();

            spawn_local(utils::r#async::stream_content(
                ipfs,
                self.content_cb.clone(),
                idx,
                regis,
            ));

            if let Some(handle) = self.stream_handle.replace(handle) {
                handle.abort();
            }
        }

        self.metadata = Some(metadata);

        true
    }

    fn on_content_discovered(&mut self, ctx: &Context<Self>, cid: Cid, media: Media) -> bool {
        let ipfs = match ctx.link().context::<IPFSContext>(Callback::noop()) {
            Some((context, _)) => context.client,
            None => return false,
        };

        if !self.identities.contains_key(&media.identity().link) {
            spawn_local(utils::r#async::dag_get(
                ipfs,
                media.identity().link,
                self.identity_cb.clone(),
            ));
        }

        self.content.push_back((cid, media));

        true
    }

    fn on_follow(&mut self, ctx: &Context<Self>) -> bool {
        let mut list = utils::follows::get_follow_list();

        if !self.subscription {
            list.insert(ctx.props().addr.into());
        } else {
            list.remove(&ctx.props().addr.into());
        }

        utils::follows::set_follow_list(list);

        self.subscription = !self.subscription;

        true
    }

    fn on_filtering(&mut self, filter: Filter) -> bool {
        if self.filter != filter {
            self.filter = filter;
            return true;
        }

        false
    }

    fn on_followees(&mut self, followees: HashMap<Cid, Identity>) -> bool {
        if self.followees != followees {
            self.followees = followees;
            return true;
        }

        false
    }
}

async fn get_followees(
    ipfs: IpfsService,
    metadata: ChannelMetadata,
    callback: Callback<HashMap<Cid, Identity>>,
) {
    let defluencer = Defluencer::new(ipfs);

    let hash_map = defluencer
        .followees_identity(std::iter::once(&metadata))
        .await;

    callback.emit(hash_map);
}
