#![cfg(target_arch = "wasm32")]

use std::collections::{HashMap, HashSet};

use cid::Cid;

use components::{pure::DagExplorer, Route};

use defluencer::channel::Channel;

use utils::{
    defluencer::{ChannelContext, UserContext},
    display_address,
    identity::{
        clear_current_identity, get_current_identity, get_identities, set_current_identity,
        set_identities,
    },
    ipfs::IPFSContext,
    web3::Web3Context,
};

use web_sys::File as SysFile;

use ybc::{
    Block, Button, ButtonRouter, Checkbox, Container, Control, Field, File, Input, Level,
    LevelItem, LevelLeft, LevelRight, Section, Subtitle, TextArea,
};

use yew::{platform::spawn_local, prelude::*};

use gloo_console::{error, info};

use linked_data::{
    identity::Identity,
    types::{Address, IPLDLink},
};

use ipfs_api::{responses::Codec, IpfsService};

#[derive(Properties, PartialEq)]
pub struct Props {
    pub context_cb: Callback<(
        Option<IPFSContext>,
        Option<Web3Context>,
        Option<UserContext>,
        Option<ChannelContext>,
    )>,
}

#[derive(PartialEq)]
pub enum Modals {
    Create,
    Import,
    Delete,
    None,
}

/// Assumes that IPFS & Web3 Context are available
pub struct IdentitySettings {
    modal: Modals,
    create_modal_cb: Callback<MouseEvent>,
    import_modal_cb: Callback<MouseEvent>,

    delete_cid: Option<Cid>,
    confirm_delete_cb: Callback<MouseEvent>,

    close_modal_cb: Callback<MouseEvent>,

    current_id: Option<IPLDLink>,
    identity_map: HashMap<Cid, Identity>,

    name: String,
    name_cb: Callback<String>,

    channel: bool,
    channel_cb: Callback<bool>,

    avatar_files: Vec<SysFile>,
    avatar_file_cb: Callback<Vec<SysFile>>,

    banner_files: Vec<SysFile>,
    banner_file_cb: Callback<Vec<SysFile>>,

    bio: String,
    bio_cb: Callback<String>,

    create_cb: Callback<MouseEvent>,
    import_cb: Callback<MouseEvent>,
    loading: bool,
}

pub enum Msg {
    Modal(Modals),
    SetID(Cid),
    DeleteID(Cid),
    Name(String),
    Channel(bool),
    Avatar(Vec<SysFile>),
    Banner(Vec<SysFile>),
    Bio(String),
    Create,
    Import,

    IdentityCreated((Cid, Identity)),
    GetIDs((Cid, Identity)),

    ConfirmDelete,
}

impl Component for IdentitySettings {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        #[cfg(debug_assertions)]
        info!("Identity Setting Create");

        let create_modal_cb = ctx.link().callback(|_| Msg::Modal(Modals::Create));
        let import_modal_cb = ctx.link().callback(|_| Msg::Modal(Modals::Import));
        let confirm_delete_cb = ctx.link().callback(|_| Msg::ConfirmDelete);
        let close_modal_cb = ctx.link().callback(|_| Msg::Modal(Modals::None));
        let name_cb = ctx.link().callback(Msg::Name);
        let channel_cb = ctx.link().callback(Msg::Channel);
        let avatar_file_cb = ctx.link().callback(Msg::Avatar);
        let banner_file_cb = ctx.link().callback(Msg::Banner);
        let create_cb = ctx.link().callback(|_| Msg::Create);
        let import_cb = ctx.link().callback(|_| Msg::Import);
        let bio_cb = ctx.link().callback(Msg::Bio);

        let current_id: Option<IPLDLink> = get_current_identity();

        let identity_set: HashSet<IPLDLink> = get_identities().unwrap_or_default();

        let cb = ctx.link().callback(Msg::GetIDs);

        if let Some((context, _)) = ctx.link().context::<IPFSContext>(Callback::noop()) {
            let ipfs = context.client;

            for identity in identity_set {
                spawn_local({
                    let cb = cb.clone();
                    let ipfs = ipfs.clone();

                    async move {
                        match ipfs.dag_get::<&str, Identity>(identity.link, None).await {
                            Ok(id) => cb.emit((identity.link, id)),
                            Err(e) => error!(&format!("{:?}", e)),
                        }
                    }
                });
            }
        }

        let mut name = String::new();

        if let Some((context, _)) = ctx.link().context::<Web3Context>(Callback::noop()) {
            if let Some(ens_name) = context.name {
                name = ens_name;
            }
        }

        Self {
            modal: Modals::None,
            create_modal_cb,
            import_modal_cb,

            delete_cid: None,
            confirm_delete_cb,

            close_modal_cb,

            current_id,
            identity_map: HashMap::new(),

            name,
            name_cb,

            channel: false,
            channel_cb,

            avatar_files: vec![],
            avatar_file_cb,

            banner_files: vec![],
            banner_file_cb,

            bio: String::new(),
            bio_cb,

            create_cb,
            import_cb,
            loading: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        #[cfg(debug_assertions)]
        info!("Identity Setting Update");

        match msg {
            Msg::Modal(modals) => self.on_modal(modals),
            Msg::SetID(cid) => self.on_set_identity(cid, ctx),
            Msg::Name(name) => self.on_name(name),
            Msg::Channel(channel) => self.on_channel(channel),
            Msg::Avatar(files) => self.on_avatar_files(files),
            Msg::Banner(files) => self.on_banner_files(files),
            Msg::Bio(text) => self.on_bio(text),
            Msg::Create => self.on_create(ctx),
            Msg::Import => self.on_import(ctx),
            Msg::IdentityCreated((cid, identity)) => self.on_identity_created(ctx, cid, identity),
            Msg::GetIDs((cid, identity)) => self.on_ids(cid, identity),
            Msg::DeleteID(cid) => self.on_delete(cid),
            Msg::ConfirmDelete => self.on_confirm_delete(ctx),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        #[cfg(debug_assertions)]
        info!("Identity Setting View");

        html! {
        <Section>
            <Container>
                <Subtitle >
                    { "Identities" }
                </Subtitle>
                { self.render_create_modal() }
                { self.render_import_modal() }
                { self.render_delete_modal() }
                <Block>
                <small>
                    {"To be searchable, add an "}
                    <a href="https://app.ens.domains" >
                    <u>{"ENS"}</u>
                    </a>
                    {" record with your channel IPNS address (ipns://bafzaajaiaejc...) as content hash."}
                </small>
                </Block>
                <Block>
                { self.render_identities(ctx) }
                </Block>
                <Level>
                    <LevelLeft>
                        <LevelItem>
                            <Button onclick={ self.create_modal_cb.clone() } >
                                { "Create Identity" }
                            </Button>
                        </LevelItem>
                        <LevelItem>
                            <Button onclick={ self.import_modal_cb.clone() } >
                                { "Import Identity" }
                            </Button>
                        </LevelItem>
                    </LevelLeft>
                </Level>
            </Container>
        </Section>
        }
    }
}

impl IdentitySettings {
    fn render_create_modal(&self) -> Html {
        html! {
        <div class= { if self.modal == Modals::Create { "modal is-active" } else { "modal" } } >
            <div class="modal-background" onclick={self.close_modal_cb.clone()} ></div>
            <div class="modal-card">
                <header class="modal-card-head">
                    <p class="modal-card-title">
                        { "Identity" }
                    </p>
                    <button class="delete" aria-label="close" onclick={self.close_modal_cb.clone()} >
                    </button>
                </header>
                <section class="modal-card-body">
                    <Field label="Name" >
                        <Control>
                            <Input name="name" value={self.name.clone()} update={self.name_cb.clone()} />
                        </Control>
                    </Field>
                    <Field label="Biography" help={"(optional)"} >
                        <Control>
                            <TextArea name="bio" value="" update={self.bio_cb.clone()} placeholder={"Add a short bio..."} rows={4} fixed_size={true} />
                        </Control>
                    </Field>
                    <Field label="Avatar" help={"Less than 1MiB, square ratio, .PNG or .JPG (optional)"} >
                        <Control>
                            <File name="avatar" files={self.avatar_files.clone()} update={self.avatar_file_cb.clone()} selector_label={"Choose an image..."} selector_icon={html!{<i class="fas fa-upload"></i>}} has_name={Some("image.jpg")} fullwidth=true />
                        </Control>
                    </Field>
                    <Field label="Create a channel?" help={"Without a channel, your content can only be accessed via direct link, CID or when shared."} >
                        <Control>
                            <Checkbox name="channel" checked={self.channel} update={self.channel_cb.clone()} />
                        </Control>
                    </Field>
                    if self.channel {
                        <Field label="Banner" help={"Less than 1MiB, 3 by 1 ratio, .PNG or .JPG (optional)"} >
                            <Control>
                                <File name="banner" files={self.banner_files.clone()} update={self.banner_file_cb.clone()} selector_label={"Choose an image..."} selector_icon={html!{<i class="fas fa-upload"></i>}} has_name={Some("image.jpg")} fullwidth=true />
                            </Control>
                        </Field>
                    }
                </section>
                <footer class="modal-card-foot">
                    <Button onclick={self.create_cb.clone()} loading={self.loading} disabled={self.name.is_empty()} >
                        { "Create" }
                    </Button>
                    <Button onclick={self.close_modal_cb.clone()}>
                        { "Cancel" }
                    </Button>
                </footer>
            </div>
        </div>
        }
    }

    fn render_import_modal(&self) -> Html {
        html! {
        <div class= { if self.modal == Modals::Import { "modal is-active" } else { "modal" } } >
            <div class="modal-background" onclick={self.close_modal_cb.clone()} ></div>
            <div class="modal-card">
                <header class="modal-card-head">
                    <p class="modal-card-title">
                        { "Identity" }
                    </p>
                    <button class="delete" aria-label="close" onclick={self.close_modal_cb.clone()} >
                    </button>
                </header>
                <section class="modal-card-body">
                    <Field label="Identity CID" >
                        <Control>
                            <Input name="cid" value="" update={self.name_cb.clone()} />
                        </Control>
                    </Field>
                </section>
                <footer class="modal-card-foot">
                    <Button onclick={self.import_cb.clone()} loading={self.loading} disabled={self.name.is_empty()} >
                        { "Import" }
                    </Button>
                    <Button onclick={self.close_modal_cb.clone()}>
                        { "Cancel" }
                    </Button>
                </footer>
            </div>
        </div>
        }
    }

    fn render_delete_modal(&self) -> Html {
        html! {
        <div class= { if self.modal == Modals::Delete { "modal is-active" } else { "modal" } } >
            <div class="modal-background" onclick={self.close_modal_cb.clone()} ></div>
            <div class="modal-card">
                <header class="modal-card-head">
                    <p class="modal-card-title">
                        { "Identity" }
                    </p>
                    <button class="delete" aria-label="close" onclick={self.close_modal_cb.clone()} >
                    </button>
                </header>
                <section class="modal-card-body">
                   { "Are you should you want to delete this identity, channel and content?" }
                </section>
                <footer class="modal-card-foot">
                    <Button onclick={self.confirm_delete_cb.clone()} >
                        { "Delete" }
                    </Button>
                    <Button onclick={self.close_modal_cb.clone()} >
                        { "Cancel" }
                    </Button>
                </footer>
            </div>
        </div>
        }
    }

    fn render_identities(&self, ctx: &Context<Self>) -> Html {
        self.identity_map
            .iter()
            .map(|(cid, identity)| {
                let cid = *cid;

                let mut disabled = false;

                if let Some(ipld) = self.current_id {
                    disabled = cid == ipld.link;
                }

                let set_cb = ctx.link().callback(move |_: MouseEvent| Msg::SetID(cid));
                let delete_cb = ctx.link().callback(move |_: MouseEvent| Msg::DeleteID(cid));

                let channel = if let Some(addr) = identity.ipns_addr {
                    html! {
                    <LevelItem>
                        <ButtonRouter<Route> route={Route::Channel { addr: addr.into() }}>
                            <span class="icon-text">
                                <span class="icon"><i class="fas fa-rss"></i></span>
                                <span> {"Go To Channel"} </span>
                            </span>
                        </ButtonRouter<Route>>
                    </LevelItem>
                    }
                } else {
                    html! {}
                };

                html! {
                <Level>
                    <LevelLeft>
                        <LevelItem>
                            { &identity.name }
                        </LevelItem>
                        <LevelItem>
                            <Button {disabled} onclick={set_cb} >
                                { "Set As Current" }
                            </Button>
                        </LevelItem>
                        { channel }
                    </LevelLeft>
                    <LevelRight>
                        <LevelItem>
                            <DagExplorer {cid} />
                        </LevelItem>
                        <LevelItem>
                            <Button onclick={delete_cb} >
                                <span class="icon is-small">
                                    <i class="fa-solid fa-trash-can"></i>
                                </span>
                            </Button>
                        </LevelItem>
                    </LevelRight>
                </Level>
                }
            })
            .collect::<Html>()
    }

    fn on_set_identity(&mut self, cid: Cid, ctx: &Context<Self>) -> bool {
        let ipfs = match ctx.link().context::<IPFSContext>(Callback::noop()) {
            Some((context, _)) => context.client,
            None => return false,
        };

        let signer = match ctx.link().context::<Web3Context>(Callback::noop()) {
            Some((context, _)) => context.signer,
            None => return false,
        };

        let identity = match self.identity_map.get(&cid) {
            Some(id) => id,
            None => {
                return false;
            }
        };

        let link: IPLDLink = cid.into();
        set_current_identity(link);
        self.current_id = Some(link);

        let user = Some(UserContext::new(ipfs.clone(), signer, cid));

        let channel = if let Some(addr) = identity.ipns_addr {
            use heck::ToSnakeCase;
            let key = identity.name.to_snake_case();

            let context = ChannelContext::new(ipfs, key, addr);

            Some(context)
        } else {
            None
        };

        ctx.props().context_cb.emit((None, None, user, channel));

        true
    }

    /// Callback when a new identity was created
    fn on_identity_created(&mut self, ctx: &Context<Self>, cid: Cid, identity: Identity) -> bool {
        let ipfs = match ctx.link().context::<IPFSContext>(Callback::noop()) {
            Some((context, _)) => context.client,
            None => return false,
        };

        let signer = match ctx.link().context::<Web3Context>(Callback::noop()) {
            Some((context, _)) => context.signer,
            None => return false,
        };

        self.loading = false;
        self.modal = Modals::None;

        let mut id_list = get_identities().unwrap_or_default();
        id_list.insert(cid.into());
        set_identities(id_list);

        self.identity_map.insert(cid, identity.clone());

        if self.current_id.is_none() {
            let link: IPLDLink = cid.into();
            set_current_identity(link);
            self.current_id = Some(link);

            let user = Some(UserContext::new(ipfs.clone(), signer, cid));

            let channel = if let Some(addr) = identity.ipns_addr {
                use heck::ToSnakeCase;
                let key = identity.name.to_snake_case();

                let context = ChannelContext::new(ipfs, key, addr);

                Some(context)
            } else {
                None
            };

            ctx.props().context_cb.emit((None, None, user, channel));
        }

        true
    }

    fn on_modal(&mut self, modals: Modals) -> bool {
        if self.modal == modals {
            return false;
        }

        self.modal = modals;

        true
    }

    fn on_name(&mut self, name: String) -> bool {
        if self.name == name {
            return false;
        }

        self.name = name;

        true
    }

    fn on_channel(&mut self, channel: bool) -> bool {
        if self.channel == channel {
            return false;
        }

        self.channel = channel;

        true
    }

    fn on_avatar_files(&mut self, files: Vec<SysFile>) -> bool {
        if self.avatar_files == files {
            return false;
        }

        self.avatar_files = files;

        true
    }

    fn on_banner_files(&mut self, files: Vec<SysFile>) -> bool {
        if self.banner_files == files {
            return false;
        }

        self.banner_files = files;

        true
    }

    fn on_bio(&mut self, text: String) -> bool {
        if self.bio == text {
            return false;
        }

        self.bio = text;

        true
    }

    fn on_create(&mut self, ctx: &Context<Self>) -> bool {
        let ipfs = match ctx.link().context::<IPFSContext>(Callback::noop()) {
            Some((context, _)) => context.client,
            None => return false,
        };

        let addr = match ctx.link().context::<Web3Context>(Callback::noop()) {
            Some((context, _)) => context.addr,
            None => return false,
        };

        spawn_local(create_identity(
            ipfs,
            self.name.clone(),
            self.bio.clone(),
            self.avatar_files.pop(),
            self.banner_files.pop(),
            self.channel,
            addr,
            ctx.link().callback(Msg::IdentityCreated),
        ));

        self.loading = true;

        true
    }

    fn on_import(&mut self, ctx: &Context<Self>) -> bool {
        let ipfs = match ctx.link().context::<IPFSContext>(Callback::noop()) {
            Some((context, _)) => context.client,
            None => return false,
        };

        let cid = match Cid::try_from(self.name.as_str()) {
            Ok(cid) => cid,
            Err(e) => {
                error!(&format!("{:?}", e));
                return false;
            }
        };

        if self.identity_map.contains_key(&cid) {
            info!("Duplicate Identity");
            self.modal = Modals::None;
            return true;
        }

        spawn_local(utils::r#async::dag_get(
            ipfs,
            cid,
            ctx.link().callback(Msg::IdentityCreated),
        ));

        self.loading = true;

        true
    }

    /// Callback when receiving identity from the list of all identities
    fn on_ids(&mut self, cid: Cid, identity: Identity) -> bool {
        if self.identity_map.insert(cid, identity).is_some() {
            return false;
        }

        true
    }

    fn on_delete(&mut self, cid: Cid) -> bool {
        if self.delete_cid == Some(cid) {
            return false;
        }

        self.delete_cid = Some(cid);
        self.modal = Modals::Delete;

        true
    }

    fn on_confirm_delete(&mut self, ctx: &Context<Self>) -> bool {
        let ipfs = match ctx.link().context::<IPFSContext>(Callback::noop()) {
            Some((context, _)) => context.client,
            None => return false,
        };

        let cid = match self.delete_cid.take() {
            Some(cid) => cid,
            None => return false,
        };

        self.modal = Modals::None;

        if self.current_id == Some(cid.into()) {
            self.current_id = None;

            clear_current_identity();
        }

        if let Some(mut id_list) = get_identities() {
            if id_list.remove(&cid.into()) {
                set_identities(id_list);
            }
        }

        if let Some(identity) = self.identity_map.remove(&cid) {
            use heck::ToSnakeCase;
            let key = identity.name.to_snake_case();

            spawn_local(delete_channel(ipfs, key));
        }

        true
    }
}

async fn create_identity(
    ipfs: IpfsService,
    name: String,
    bio: String,
    avatar_file: Option<SysFile>,
    banner_file: Option<SysFile>,
    channel: bool,
    eth_addr: Address,
    cb: Callback<(Cid, Identity)>,
) {
    let (avatar, banner) = match (avatar_file, banner_file) {
        (Some(avatar), Some(banner)) => {
            let (ava_res, bann_res) = futures_util::join!(
                defluencer::utils::add_image(&ipfs, avatar),
                defluencer::utils::add_image(&ipfs, banner)
            );

            let avatar = match ava_res {
                Ok(cid) => Some(cid.into()),
                Err(e) => {
                    error!(&format!("{:?}", e));
                    None
                }
            };

            let banner = match bann_res {
                Ok(cid) => Some(cid.into()),
                Err(e) => {
                    error!(&format!("{:?}", e));
                    None
                }
            };

            (avatar, banner)
        }
        (None, Some(banner)) => match defluencer::utils::add_image(&ipfs, banner).await {
            Ok(cid) => (None, Some(cid.into())),
            Err(e) => {
                error!(&format!("{:?}", e));
                (None, None)
            }
        },
        (Some(avatar), None) => match defluencer::utils::add_image(&ipfs, avatar).await {
            Ok(cid) => (Some(cid.into()), None),
            Err(e) => {
                error!(&format!("{:?}", e));
                (None, None)
            }
        },
        _ => (None, None),
    };

    let bio = if bio.is_empty() { None } else { Some(bio) };

    let mut identity = Identity {
        name,
        bio,
        avatar,
        banner,
        eth_addr: Some(display_address(eth_addr)),
        ..Default::default()
    };

    let cid = match ipfs.dag_put(&identity, Codec::default()).await {
        Ok(cid) => cid,
        Err(e) => {
            error!(&format!("{:?}", e));
            return;
        }
    };

    if channel {
        let (channel, cid) = match Channel::create_local(ipfs.clone(), cid).await {
            Ok(tuple) => tuple,
            Err(e) => {
                error!(&format!("{:?}", e));
                return;
            }
        };

        identity.ipns_addr = Some(channel.get_address());

        cb.emit((cid, identity));

        return;
    }

    cb.emit((cid, identity));
}

async fn delete_channel(ipfs: IpfsService, key: String) {
    let key_list = match ipfs.key_list().await {
        Ok(list) => list,
        Err(e) => {
            error!(&format!("{:?}", e));
            return;
        }
    };

    let addr = match key_list.get(&key) {
        Some(addr) => *addr,
        None => return,
    };

    let cid = match ipfs.name_resolve(addr).await {
        Ok(cid) => cid,
        Err(e) => {
            error!(&format!("{:?}", e));
            return;
        }
    };

    if let Err(e) = ipfs.pin_rm(cid, true).await {
        error!(&format!("{:?}", e));
    }

    if let Err(e) = ipfs.key_rm(key).await {
        error!(&format!("{:?}", e));
    }
}
