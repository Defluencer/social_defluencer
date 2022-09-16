#![cfg(target_arch = "wasm32")]

use std::collections::{HashMap, HashSet};

use cid::Cid;

use components::{dag_explorer::DagExplorer, navbar::Route};

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
    Button, ButtonRouter, Checkbox, Container, Control, Field, File, Input, Level, LevelItem,
    LevelLeft, LevelRight, Section, Subtitle,
};

use yew::prelude::*;

use gloo_console::{error, info};

use wasm_bindgen_futures::spawn_local;

use linked_data::{
    identity::Identity,
    types::{Address, IPLDLink},
};

use ipfs_api::{responses::Codec, IpfsService};

#[derive(Properties, PartialEq)]
pub struct Props {
    pub user_cb: Callback<UserContext>,
    pub channel_cb: Callback<ChannelContext>,
}

#[derive(PartialEq)]
pub enum Modals {
    Create,
    Delete,
    None,
}

/// Assumes that IPFS & Web3 Context are available
pub struct IdentitySettings {
    modal: Modals,
    create_modal_cb: Callback<MouseEvent>,

    delete_cid: Option<Cid>,
    confirm_delete_cb: Callback<MouseEvent>,

    close_modal_cb: Callback<MouseEvent>,

    current_id: Option<IPLDLink>,
    identity_map: HashMap<Cid, Identity>,

    name: String,
    name_cb: Callback<String>,

    channel: bool,
    channel_cb: Callback<bool>,

    files: Vec<SysFile>,
    file_cb: Callback<Vec<SysFile>>,

    identity_cb: Callback<MouseEvent>,
    loading: bool,
}

pub enum Msg {
    Modal(Modals),
    SetID(Cid),
    DeleteID(Cid),
    Name(String),
    Channel(bool),
    Files(Vec<SysFile>),
    Create,

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

        let current_id: Option<IPLDLink> = get_current_identity();

        let identity_set: HashSet<IPLDLink> = get_identities().unwrap_or_default();

        let (context, _) = ctx
            .link()
            .context::<IPFSContext>(Callback::noop())
            .expect("IPFS Context");
        let ipfs = context.client;

        let cb = ctx.link().callback(Msg::GetIDs);

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

        let create_modal_cb = ctx.link().callback(|_| Msg::Modal(Modals::Create));

        let confirm_delete_cb = ctx.link().callback(|_| Msg::ConfirmDelete);

        let close_modal_cb = ctx.link().callback(|_| Msg::Modal(Modals::None));

        let name_cb = ctx.link().callback(Msg::Name);
        let channel_cb = ctx.link().callback(Msg::Channel);
        let file_cb = ctx.link().callback(Msg::Files);
        let identity_cb = ctx.link().callback(|_| Msg::Create);

        Self {
            modal: Modals::None,
            create_modal_cb,

            delete_cid: None,
            confirm_delete_cb,

            close_modal_cb,

            current_id,
            identity_map: HashMap::new(),

            name: String::new(),
            name_cb,

            channel: false,
            channel_cb,

            files: vec![],
            file_cb,

            identity_cb,
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
            Msg::Files(files) => self.on_files(files),
            Msg::Create => self.on_create(ctx),
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
                { self.render_delete_modal() }
                { self.render_identities(ctx) }
                <Button onclick={ self.create_modal_cb.clone() } >
                    { "Create New Identity" }
                </Button>
            </Container>
        </Section>
        }
    }
}

impl IdentitySettings {
    fn render_create_modal(&self) -> Html {
        html! {
        <div class= { if self.modal == Modals::Create { "modal is-active" } else { "modal" } } >
            <div class="modal-background"></div>
            <div class="modal-card">
                <header class="modal-card-head">
                    <p class="modal-card-title">
                        { "Identity" }
                    </p>
                    <button class="delete" aria-label="close" onclick={self.close_modal_cb.clone()} >
                    </button>
                </header>
                <section class="modal-card-body">
                    <Field label="Display name" >
                        <Control>
                            <Input name="name" value="" update={self.name_cb.clone()} />
                        </Control>
                    </Field>
                    <Field label="Create a channel too?" help={"Takes ~2 minutes to publish a new channel"} >
                        <Control>
                            <Checkbox name="channel" checked=false update={self.channel_cb.clone()} />
                        </Control>
                    </Field>
                    <Field label="Avatar" >
                        <Control>
                            <File name="avatar" files={self.files.clone()} update={self.file_cb.clone()} selector_label={"Choose an image..."} selector_icon={html!{<i class="fas fa-upload"></i>}} has_name={Some("image.jpg")} fullwidth=true />
                        </Control>
                    </Field>
                </section>
                <footer class="modal-card-foot">
                    <Button onclick={self.identity_cb.clone()} loading={self.loading} disabled={self.files.is_empty()} >
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

    fn render_delete_modal(&self) -> Html {
        html! {
        <div class= { if self.modal == Modals::Delete { "modal is-active" } else { "modal" } } >
            <div class="modal-background"></div>
            <div class="modal-card">
                <header class="modal-card-head">
                    <p class="modal-card-title">
                        { "Identity" }
                    </p>
                    <button class="delete" aria-label="close" onclick={self.close_modal_cb.clone()} >
                    </button>
                </header>
                <section class="modal-card-body">
                   { "Are you should you want to delete this identity?" }
                </section>
                <footer class="modal-card-foot">
                    <Button onclick={self.confirm_delete_cb.clone()} >
                        { "Delete" }
                    </Button>
                    <Button onclick={self.close_modal_cb.clone()}>
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

                let disabled = self.current_id.is_some() && cid == self.current_id.unwrap().link;

                let set_cb = ctx.link().callback(move |_: MouseEvent| Msg::SetID(cid));
                let delete_cb = ctx.link().callback(move |_: MouseEvent| Msg::DeleteID(cid));

                let channel = if let Some(addr) = identity.channel_ipns {
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
                            { &identity.display_name }
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
        let link: IPLDLink = cid.into();
        set_current_identity(link);
        self.current_id = Some(link);

        if let Some(identity) = self.identity_map.get(&cid) {
            let (context, _) = ctx
                .link()
                .context::<IPFSContext>(Callback::noop())
                .expect("IPFS Context");
            let ipfs = context.client;

            let (web3_context, _) = ctx
                .link()
                .context::<Web3Context>(Callback::noop())
                .expect("Web3 Context");
            let signer = web3_context.signer;

            ctx.props()
                .user_cb
                .emit(UserContext::new(ipfs.clone(), signer, cid));

            if let Some(addr) = identity.channel_ipns {
                use heck::ToSnakeCase;
                let key = identity.display_name.to_snake_case();

                let context = ChannelContext::new(ipfs, key, addr);

                ctx.props().channel_cb.emit(context);
            }
        }

        true
    }

    /// Callback when a new identity was created
    fn on_identity_created(&mut self, ctx: &Context<Self>, cid: Cid, identity: Identity) -> bool {
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

            let (context, _) = ctx
                .link()
                .context::<IPFSContext>(Callback::noop())
                .expect("IPFS Context");
            let ipfs = context.client;

            let (web3_context, _) = ctx
                .link()
                .context::<Web3Context>(Callback::noop())
                .expect("Web3 Context");
            let signer = web3_context.signer;

            ctx.props()
                .user_cb
                .emit(UserContext::new(ipfs.clone(), signer, cid));

            if let Some(addr) = identity.channel_ipns {
                use heck::ToSnakeCase;
                let key = identity.display_name.to_snake_case();

                let context = ChannelContext::new(ipfs, key, addr);

                ctx.props().channel_cb.emit(context);
            }
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

    fn on_files(&mut self, files: Vec<SysFile>) -> bool {
        if self.files == files {
            return false;
        }

        self.files = files;

        true
    }

    fn on_create(&mut self, ctx: &Context<Self>) -> bool {
        let (context, _) = ctx
            .link()
            .context::<IPFSContext>(Callback::noop())
            .expect("IPFS Context");
        let ipfs = context.client;

        let (context, _) = ctx
            .link()
            .context::<Web3Context>(Callback::noop())
            .expect("Web3 Context");
        let addr = context.addr;

        spawn_local(create_identity(
            ipfs,
            self.files.pop().unwrap(),
            self.channel,
            self.name.clone(),
            addr,
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
            let key = identity.display_name.to_snake_case();

            let (context, _) = ctx
                .link()
                .context::<IPFSContext>(Callback::noop())
                .expect("IPFS Context");
            let ipfs = context.client;

            spawn_local({
                async move {
                    if let Err(e) = ipfs.key_rm(key).await {
                        error!(&format!("{:?}", e));
                    }
                }
            });
        }

        true
    }
}

async fn create_identity(
    ipfs: IpfsService,
    file: SysFile,
    channel: bool,
    display_name: String,
    addr: Address,
    cb: Callback<(Cid, Identity)>,
) {
    let avatar = match defluencer::utils::add_image(&ipfs, file).await {
        Ok(cid) => Some(cid.into()),
        Err(e) => {
            error!(&format!("{:?}", e));
            return;
        }
    };

    let addr = Some(display_address(addr));

    let mut identity = Identity {
        display_name,
        avatar,
        channel_ipns: None,
        addr,
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

        identity.channel_ipns = Some(channel.get_address());

        cb.emit((cid, identity));

        return;
    }

    cb.emit((cid, identity));
}
