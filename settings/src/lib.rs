use yew::{context::ContextHandle, prelude::*};

use utils::{ipfs::IPFSContext, web3::Web3Context};

use linked_data::types::PeerId;

use web_sys::{EventTarget, HtmlInputElement};

use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

use gloo_console::{error, info};
use gloo_storage::{LocalStorage, Storage};

use ipfs_api::DEFAULT_URI;

const IPFS_API_ADDRS_KEY: &str = "ipfs_api_addrs";

#[derive(Properties, PartialEq)]
pub struct Props {
    pub ipfs_cb: Callback<IPFSContext>,
    pub web3_cb: Callback<Web3Context>,
}

pub struct SettingPage {
    peer_id: Option<PeerId>,
    _context_handle: Option<ContextHandle<IPFSContext>>,

    address: String,
    address_cb: Callback<Event>,

    os_type: OsType,
    win_cb: Callback<MouseEvent>,
    unix_cb: Callback<MouseEvent>,

    origin: String,
}

pub enum Msg {
    PeerId(PeerId),
    Addrs(String),
    OsType(OsType),
}

// social.defluencer.eth/#/settings/
// User & channels section for switching identity or publishing channel.
// IPFS section, changes when connected

impl Component for SettingPage {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        #[cfg(debug_assertions)]
        info!("Setting Create");

        let peer_id_cb = ctx
            .link()
            .callback(|context: IPFSContext| Msg::PeerId(context.peer_id));

        let (peer_id, _context_handle) = match ctx.link().context::<IPFSContext>(peer_id_cb.clone())
        {
            Some((context, handle)) => (Some(context.peer_id), Some(handle)),
            None => (None, None),
        };

        let address = LocalStorage::get(IPFS_API_ADDRS_KEY).unwrap_or(DEFAULT_URI.to_owned());

        let address_cb = ctx.link().callback(|e: Event| {
            let target: EventTarget = e
                .target()
                .expect("Event should have a target when dispatched");

            Msg::Addrs(target.unchecked_into::<HtmlInputElement>().value())
        });

        let win_cb = ctx
            .link()
            .callback(|_event: MouseEvent| Msg::OsType(OsType::Windows));

        let unix_cb = ctx
            .link()
            .callback(|_event: MouseEvent| Msg::OsType(OsType::Unix));

        let origin = {
            let mut temp = Default::default();

            if let Some(win) = web_sys::window() {
                match win.location().origin() {
                    Ok(org) => temp = org,
                    Err(e) => error!(&format!("{:?}", e)),
                }
            }

            temp
        };

        Self {
            peer_id,
            _context_handle,

            address,
            address_cb,

            os_type: OsType::Unix,
            win_cb,
            unix_cb,

            origin,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        #[cfg(debug_assertions)]
        info!("Setting Update");

        match msg {
            Msg::PeerId(peer) => {
                self.peer_id = Some(peer);

                true
            }
            Msg::Addrs(msg) => {
                if msg != self.address {
                    if let Err(e) = LocalStorage::set(IPFS_API_ADDRS_KEY, &msg) {
                        error!(&format!("{:?}", e));
                    }

                    spawn_local({
                        let cb = ctx.props().ipfs_cb.clone();
                        let url = msg.clone();

                        async move {
                            if let Some(context) = IPFSContext::new(Some(url)).await {
                                cb.emit(context);
                            }
                        }
                    });

                    self.address = msg;
                }

                true
            }
            Msg::OsType(os_type) => {
                if self.os_type != os_type {
                    self.os_type = os_type;

                    return true;
                }

                false
            } //Msg::NodeType(msg) => self.on_node_type(msg),
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        #[cfg(debug_assertions)]
        info!("Setting View");

        html! {
        <>
            //<Navbar />
            <ybc::Section>
                <ybc::Container>
                    {
                        match self.peer_id {
                            Some(peer_id) => self.render_connected(peer_id),
                            None => self.render_not_connected(),
                        }
                    }
                    /* <div class="field">
                        <label class="label"> { "IPFS Node" } </label>
                        <div class="control is-expanded">
                            <div class="select is-fullwidth">
                                <select id="node_type" onchange=self.node_cb.clone() >
                                    <option selected=brave_slct value="Brave"> { "Brave" } </option>
                                    <option selected=ext_slct value="External"> { "External" } </option>
                                </select>
                            </div>
                        </div>
                        <p class="help"> { "External nodes can be configured for better performace but Brave browser nodes are more conveniant." } </p>
                    </div> */
                    <div class="field">
                        <label class="label"> { "IPFS API" } </label>
                        <div class="control is-expanded">
                            <input name="ipfs_addrs" value={self.address.clone()} onchange={self.address_cb.clone()} class="input" type="text" />
                        </div>
                        <p class="help"> { "Refresh to apply changes." } </p>
                    </div>
                </ybc::Container>
            </ybc::Section>
        </>
        }
    }
}

impl SettingPage {
    fn render_connected(&self, peer_id: PeerId) -> Html {
        html! {
            <div class="field">
                <label class="label"> { "IPFS Peer ID" } </label>
                <div class="control is-expanded">
                    <input name="ipfs_addrs" value={peer_id.to_string()} class="input is-static" type="text" readonly=true />
                </div>
                <p class="help"> { "A unique string identifing this node on the network." } </p>
            </div>
        }
    }

    fn render_code(&self) -> Html {
        let (deliminator, separator) = match self.os_type {
            OsType::Unix => (r#"'"#, r#"""#),
            OsType::Windows => (r#"""#, r#"""""#),
        };

        html! {
            <div style="white-space: nowrap;overflow-x: auto;overflow-y: hidden;">
                <code style="display: block"> { format!(r#"ipfs config --json API.HTTPHeaders.Access-Control-Allow-Methods {delim}[{sep}POST{sep}]{delim}"#, sep = separator, delim = deliminator) } </code>
                <code style="display: block"> { format!(r#"ipfs config --json API.HTTPHeaders.Access-Control-Allow-Origin {delim}[{sep}https://webui.ipfs.io{sep}, {sep}http://127.0.0.1:5001{sep}, {sep}{url}{sep}]{delim}"#, sep = separator, delim = deliminator, url = self.origin) } </code>
            </div>
        }
    }

    fn render_not_connected(&self) -> Html {
        /* let port = if self.node_type == NodeType::Brave {
            "45005"
        } else {
            "5001"
        }; */

        html! {
            <>
                <ybc::Block>
                <span class="icon-text">
                    <span class="icon is-large has-text-danger"><i class="fas fa-exclamation-triangle fa-3x"></i></span>
                    <span class="title"> { "Cannot connect to IPFS" } </span>
                </span>
                </ybc::Block>
                <ybc::Block>
                <h2 class="subtitle">
                    { "Follow the installation guide in the " }
                    <a href="https://docs.ipfs.io/how-to/command-line-quick-start/"> { "IPFS Documentation" } </a>
                    { " or configure your node correctly." }
                </h2>
                </ybc::Block>
                <ybc::Block>
                <ol>
                    <li>
                        <p>{ "Is your IPFS daemon running? Start your daemon with the terminal command below." }</p>
                        <div style="white-space: nowrap;overflow-x: auto;overflow-y: hidden;">
                            <code style="display: block"> { "ipfs daemon --enable-pubsub-experiment --enable-namesys-pubsub" } </code>
                        </div>
                    </li>
                    <li>
                        <p>
                            {"Is your IPFS API configured to allow "}
                            <a href="https://github.com/ipfs-shipyard/ipfs-webui#configure-ipfs-api-cors-headers">
                                {"cross-origin (CORS) requests"}
                            </a>
                            {"? If not, run these terminal commands and restart your daemon."}
                        </p>
                        <ybc::Tabs classes={classes!("is-small")} >
                            <li class={if let OsType::Unix = self.os_type {"is-active"} else {""}} >
                                <a onclick={self.unix_cb.clone()} >
                                    <span class="icon-text">
                                        <span class="icon"><i class="fab fa-linux"></i></span>
                                        <span> { "Linux" } </span>
                                    </span>
                                    <span class="icon-text">
                                        <span class="icon"><i class="fab fa-apple"></i></span>
                                        <span> { "MacOs" } </span>
                                    </span>
                                </a>
                            </li>
                            <li class={if let OsType::Windows = self.os_type {"is-active"} else {""}} >
                                <a onclick={self.win_cb.clone()} >
                                    <span class="icon-text">
                                        <span class="icon"><i class="fab fa-windows"></i></span>
                                        <span> { "Windows" } </span>
                                    </span>
                                </a>
                            </li>
                        </ybc::Tabs>
                        { self.render_code() }
                    </li>
                </ol>
                </ybc::Block>
            </>
        }
    }
}

#[derive(PartialEq)]
pub enum OsType {
    Unix,
    Windows,
}
