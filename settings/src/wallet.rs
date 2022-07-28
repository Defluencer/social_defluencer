use linked_data::types::Address;
use yew::{context::ContextHandle, prelude::*};

use utils::web3::{get_wallet_addr, set_wallet_addr, Web3Context};

use web_sys::{EventTarget, HtmlInputElement};

use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

use gloo_console::{error, info};

use gloo_storage::{LocalStorage, Storage};

use cid::Cid;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub web3_cb: Callback<Web3Context>,
}

pub struct WalletSettings {
    address: Option<Address>,
    _context_handle: Option<ContextHandle<Web3Context>>,

    wallet: String,
    wallet_cb: Callback<MouseEvent>,
}

pub enum Msg {
    EthAddr(Address),
    ConnectWallet,
}

impl Component for WalletSettings {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        #[cfg(debug_assertions)]
        info!("Wallet Setting Create");

        let address_cb = ctx
            .link()
            .callback(|context: Web3Context| Msg::EthAddr(context.addr));

        let (address, _context_handle) = match ctx.link().context::<Web3Context>(address_cb.clone())
        {
            Some((context, handle)) => (Some(context.addr), Some(handle)),
            None => (None, None),
        };

        let wallet = get_wallet_addr().unwrap_or_default();
        let wallet_cb = ctx.link().callback(|_e: MouseEvent| Msg::ConnectWallet);

        Self {
            address,
            _context_handle,

            wallet,
            wallet_cb,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        #[cfg(debug_assertions)]
        info!("Wallet Setting Update");

        match msg {
            Msg::ConnectWallet => {
                //TODO

                spawn_local({
                    let cb = ctx.props().web3_cb.clone();

                    async move {
                        if let Some(context) = Web3Context::new().await {
                            cb.emit(context);
                        }
                    }
                });

                true
            }
            Msg::EthAddr(addr) => {
                self.address = Some(addr);

                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        #[cfg(debug_assertions)]
        info!("Wallet Setting View");

        html! {
        <ybc::Section>
            <ybc::Container>
                {
                    match self.address {
                        Some(addr) => self.render_connected(addr),
                        None => self.render_unconnected(),
                    }
                }
            </ybc::Container>
        </ybc::Section>
        }
    }
}

impl WalletSettings {
    fn render_connected(&self, addr: Address) -> Html {
        html! {}
    }

    fn render_unconnected(&self) -> Html {
        html! {}
    }
}
