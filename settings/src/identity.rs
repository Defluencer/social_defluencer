#![cfg(target_arch = "wasm32")]

use std::collections::{HashMap, HashSet};

use cid::Cid;

use components::cid_explorer::CidExplorer;

use utils::{
    identity::{
        clear_current_identity, get_current_identity, get_identities, set_current_identity,
        set_identities,
    },
    ipfs::IPFSContext,
};

use web_sys::File as SysFile;

use ybc::{
    Button, Checkbox, Container, Control, Field, File, Input, Level, LevelItem, LevelLeft,
    LevelRight, Section, Subtitle,
};

use yew::prelude::*;

use gloo_console::{error, info};

use wasm_bindgen_futures::spawn_local;

use linked_data::{
    identity::Identity,
    types::{IPLDLink, IPNSAddress},
};

use ipfs_api::{responses::Codec, IpfsService};

pub struct IdentitySettings {
    pub modal: bool,
    pub modal_cb: Callback<MouseEvent>,

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
    Modal,
    SetID(Cid),
    DeleteID(Cid),
    Name(String),
    Channel(bool),
    Files(Vec<SysFile>),
    Create,

    Done((Cid, Identity)),
    GetIDs((Cid, Identity)),
}

impl Component for IdentitySettings {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        #[cfg(debug_assertions)]
        info!("Identity Setting Create");

        let current_id: Option<IPLDLink> = get_current_identity();
        let ipld_set: HashSet<IPLDLink> = get_identities().unwrap_or_default();

        let (context, _) = ctx
            .link()
            .context::<IPFSContext>(Callback::noop())
            .expect("IPFS Context");
        let ipfs = context.client;

        let cb = ctx.link().callback(Msg::GetIDs);

        for ipld in ipld_set {
            spawn_local({
                let cb = cb.clone();
                let ipfs = ipfs.clone();

                async move {
                    match ipfs.dag_get::<String, Identity>(ipld.link, None).await {
                        Ok(id) => cb.emit((ipld.link, id)),
                        Err(e) => error!(&format!("{:?}", e)),
                    }
                }
            });
        }

        let modal_cb = ctx.link().callback(|_| Msg::Modal);
        let name_cb = ctx.link().callback(Msg::Name);
        let channel_cb = ctx.link().callback(Msg::Channel);
        let file_cb = ctx.link().callback(Msg::Files);
        let identity_cb = ctx.link().callback(|_| Msg::Create);

        Self {
            modal: false,
            modal_cb,

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
            Msg::Modal => {
                self.modal = !self.modal;

                true
            }
            Msg::SetID(cid) => self.set_current_identity(cid),
            Msg::Name(name) => {
                if name.is_empty() {
                    return false;
                }

                self.name = name;

                true
            }
            Msg::Channel(channel) => {
                self.channel = channel;

                true
            }
            Msg::Files(files) => {
                if files.is_empty() {
                    return false;
                }

                self.files = files;

                true
            }
            Msg::Create => {
                let (context, _) = ctx
                    .link()
                    .context::<IPFSContext>(Callback::noop())
                    .expect("IPFS Context");

                self.loading = true;

                spawn_local(create_identity(
                    context.client.clone(),
                    self.files.pop().unwrap(),
                    self.channel,
                    self.name.clone(),
                    ctx.link().callback(Msg::Done),
                ));

                true
            }

            Msg::Done((cid, identity)) => {
                self.loading = false;
                self.modal = false;

                let mut id_list = get_identities().unwrap_or_default();
                id_list.insert(cid.into());
                set_identities(id_list);

                self.identity_map.insert(cid, identity);

                if self.current_id.is_none() {
                    self.set_current_identity(cid);
                }

                true
            }
            Msg::GetIDs((cid, identity)) => {
                self.identity_map.insert(cid, identity);

                true
            }
            Msg::DeleteID(cid) => {
                let (context, _) = ctx
                    .link()
                    .context::<IPFSContext>(Callback::noop())
                    .expect("IPFS Context");

                self.identity_map.remove(&cid);

                if self.current_id.is_some() && self.current_id.unwrap().link == cid {
                    self.current_id = None;

                    clear_current_identity();
                }

                if let Some(mut id_list) = get_identities() {
                    id_list.remove(&cid.into());

                    set_identities(id_list);
                }

                spawn_local({
                    let ipfs = context.client.clone();

                    async move {
                        if let Err(e) = ipfs.pin_rm(cid, true).await {
                            error!(&format!("{:?}", e));
                        }

                        //TODO key remove?

                        // once IPNS records are created with crypto wallet
                        // Key in IPFS won't be needed
                    }
                });

                true
            }
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
                { self.modal() }
                { self.identity_display(ctx) }
                <Button onclick={ self.modal_cb.clone() } >
                    { "Create New Identity" }
                </Button>
            </Container>
        </Section>
        }
    }
}

impl IdentitySettings {
    fn modal(&self) -> Html {
        html! {
        <div class= { if self.modal { "modal is-active" } else { "modal" } } >
            <div class="modal-background"></div>
            <div class="modal-card">
                <header class="modal-card-head">
                    <p class="modal-card-title">
                        { "New Identity" }
                    </p>
                    <button class="delete" aria-label="close" onclick={self.modal_cb.clone()} >
                    </button>
                </header>
                <section class="modal-card-body">
                    <Field label="Display Name" >
                        <Control>
                            <Input name="name" value="" update={self.name_cb.clone()} />
                        </Control>
                    </Field>
                    <Field label="Create Channel ?" >
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
                        { "Create New identity" }
                    </Button>
                    <Button onclick={self.modal_cb.clone()}>
                        { "Cancel" }
                    </Button>
                </footer>
            </div>
        </div>
        }
    }

    fn identity_display(&self, ctx: &Context<Self>) -> Html {
        self.identity_map
            .iter()
            .map(|(cid, identity)| {
                let cid = *cid;

                let disabled = self.current_id.is_some() && cid == self.current_id.unwrap().link;

                let set_cb = ctx.link().callback(move |_: MouseEvent| Msg::SetID(cid));
                let delete_cb = ctx.link().callback(move |_: MouseEvent| Msg::DeleteID(cid));

                html! {
                <Level>
                    <LevelLeft>
                        <LevelItem>
                            <Button {disabled} onclick={set_cb} >
                                { "Set As Current" }
                            </Button>
                        </LevelItem>
                        <LevelItem>
                            { &identity.display_name }
                        </LevelItem>
                        <LevelItem>
                            //TODO channel link
                        </LevelItem>
                        <LevelItem>
                            <CidExplorer {cid} />
                        </LevelItem>
                    </LevelLeft>
                    <LevelRight>
                        <Button onclick={delete_cb} >
                            <span class="icon is-small">
                                <i class="fa-solid fa-trash-can"></i>
                            </span>
                        </Button>
                    </LevelRight>
                </Level>
                }
            })
            .collect::<Html>()
    }

    fn set_current_identity(&mut self, cid: Cid) -> bool {
        let link: IPLDLink = cid.into();

        set_current_identity(link);

        self.current_id = Some(link);

        true
    }
}

async fn create_identity(
    ipfs: IpfsService,
    file: SysFile,
    channel: bool,
    display_name: String,
    cb: Callback<(Cid, Identity)>,
) {
    let avatar = match defluencer::utils::add_image(&ipfs, file).await {
        Ok(cid) => Some(cid.into()),
        Err(e) => {
            error!(&format!("{:?}", e));
            return;
        }
    };

    let mut channel_ipns = None;

    if channel {
        use heck::ToSnakeCase;

        let key = display_name.to_snake_case();

        let key_pair = match ipfs.key_gen(key).await {
            Ok(kp) => kp,
            Err(e) => {
                error!(&format!("{:?}", e));
                return;
            }
        };

        let addr = match IPNSAddress::try_from(key_pair.id.as_str()) {
            Ok(addr) => addr,
            Err(e) => {
                error!(&format!("{:?}", e));
                return;
            }
        };

        channel_ipns = Some(addr);
    }

    let identity = Identity {
        display_name,
        avatar,
        channel_ipns,
    };

    let cid = match ipfs.dag_put(&identity, Codec::default()).await {
        Ok(cid) => cid,
        Err(e) => {
            error!(&format!("{:?}", e));
            return;
        }
    };

    match ipfs.pin_add(cid, true).await {
        Ok(_) => cb.emit((cid, identity)),
        Err(e) => error!(&format!("{:?}", e)),
    }
}
