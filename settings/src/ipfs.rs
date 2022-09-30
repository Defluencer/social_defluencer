#![cfg(target_arch = "wasm32")]

use ybc::{Block, Container, Section, Subtitle, Tabs};

use yew::{context::ContextHandle, platform::spawn_local, prelude::*};

use utils::{
    defluencer::{ChannelContext, UserContext},
    ipfs::{get_ipfs_addr, set_ipfs_addr, IPFSContext},
    web3::Web3Context,
};

use web_sys::{EventTarget, HtmlInputElement};

use wasm_bindgen::JsCast;

use gloo_console::{error, info};

use cid::Cid;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub context_cb: Callback<(
        Option<IPFSContext>,
        Option<Web3Context>,
        Option<UserContext>,
        Option<ChannelContext>,
    )>,
}

pub struct IPFSSettings {
    peer_id: Option<Cid>,
    _context_handle: Option<ContextHandle<IPFSContext>>,

    address: String,
    address_cb: Callback<Event>,

    os_type: OsType,
    win_cb: Callback<MouseEvent>,
    unix_cb: Callback<MouseEvent>,

    origin: String,
}

pub enum Msg {
    //PeerId(Cid),
    Addrs(String),
    OsType(OsType),
}

impl Component for IPFSSettings {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        #[cfg(debug_assertions)]
        info!("IPFS Setting Create");

        let (peer_id, _context_handle) = match ctx.link().context::<IPFSContext>(Callback::noop()) {
            Some((context, handle)) => (Some(context.peer_id.into()), Some(handle)),
            None => (None, None),
        };

        let address = get_ipfs_addr();

        let address_cb = ctx.link().callback(|e: Event| {
            let target: EventTarget = e
                .target()
                .expect("Event should have a target when dispatched");

            Msg::Addrs(target.unchecked_into::<HtmlInputElement>().value())
        });

        let win_cb = ctx
            .link()
            .callback(|_e: MouseEvent| Msg::OsType(OsType::Windows));

        let unix_cb = ctx
            .link()
            .callback(|_e: MouseEvent| Msg::OsType(OsType::Unix));

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
        info!("IPFS Setting Update");

        match msg {
            /* Msg::PeerId(peer) => {
                self.peer_id = Some(peer);

                true
            } */
            Msg::Addrs(msg) => {
                if msg != self.address {
                    spawn_local({
                        let cb = ctx.props().context_cb.clone();
                        let url = msg.clone();
                        let addr = msg.clone();

                        async move {
                            if let Some(context) = IPFSContext::new(Some(&url)).await {
                                set_ipfs_addr(&addr);

                                cb.emit((Some(context), None, None, None));
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
        info!("IPFS Setting View");

        html! {
        <Section>
            <Container>
                <Subtitle >
                    {"InterPlanetary File System"}
                </Subtitle>
                {
                    match self.peer_id {
                        Some(peer_id) => self.render_connected(peer_id),
                        None => self.render_unconnected(),
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
                    <label class="label"> { "API" } </label>
                    <div class="control is-expanded">
                        <input name="ipfs_addrs" value={self.address.clone()} onchange={self.address_cb.clone()} class="input" type="text" />
                    </div>
                    <p class="help"> { "Refresh to apply changes." } </p>
                </div>
            </Container>
        </Section>
        }
    }
}

impl IPFSSettings {
    /// Os dependent render of console commands.
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

    fn render_connected(&self, peer_id: Cid) -> Html {
        html! {
            <div class="field">
                <label class="label"> { "Peer ID" } </label>
                <div class="control is-expanded">
                    <input name="ipfs_addrs" value={peer_id.to_string()} class="input is-static" type="text" readonly=true />
                </div>
                <p class="help"> { "A unique string identifing this node on the network." } </p>
            </div>
        }
    }

    fn render_unconnected(&self) -> Html {
        /* let port = if self.node_type == NodeType::Brave {
            "45005"
        } else {
            "5001"
        }; */

        html! {
            <>
                <Block>
                <span class="icon-text">
                    <span class="icon is-large has-text-danger"><i class="fas fa-exclamation-triangle fa-3x"></i></span>
                    <span class="title"> { "Cannot connect to IPFS" } </span>
                </span>
                </Block>
                <Block>
                <h2 class="subtitle">
                    { "Follow the installation guide in the " }
                    <a href="https://docs.ipfs.io/how-to/command-line-quick-start/"> { "IPFS Documentation" } </a>
                    { " or configure your node correctly." }
                </h2>
                </Block>
                <Block>
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
                        <Tabs classes={classes!("is-small")} >
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
                        </Tabs>
                        { self.render_code() }
                    </li>
                </ol>
                </Block>
            </>
        }
    }
}

#[derive(PartialEq)]
pub enum OsType {
    Unix,
    Windows,
}
