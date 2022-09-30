#![cfg(target_arch = "wasm32")]

use linked_data::types::Address;

use ybc::{Button, Container, Section, Subtitle};

use yew::{context::ContextHandle, prelude::*};

use utils::{
    defluencer::{ChannelContext, UserContext},
    display_address,
    ipfs::IPFSContext,
    web3::{set_wallet_addr, Web3Context},
};

use wasm_bindgen_futures::spawn_local;

use gloo_console::info;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub context_cb: Callback<(
        Option<IPFSContext>,
        Option<Web3Context>,
        Option<UserContext>,
        Option<ChannelContext>,
    )>,
}

pub struct WalletSettings {
    address: Option<Address>,
    _context_handle: Option<ContextHandle<Web3Context>>,

    wallet_cb: Callback<MouseEvent>,
}

pub enum Msg {
    //EthAddr(Address),
    ConnectWallet,
}

impl Component for WalletSettings {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        #[cfg(debug_assertions)]
        info!("Wallet Setting Create");

        let (address, _context_handle) = match ctx.link().context::<Web3Context>(Callback::noop()) {
            Some((context, handle)) => (Some(context.addr), Some(handle)),
            None => (None, None),
        };

        let wallet_cb = ctx.link().callback(|_e: MouseEvent| Msg::ConnectWallet);

        Self {
            address,
            _context_handle,

            wallet_cb,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        #[cfg(debug_assertions)]
        info!("Wallet Setting Update");

        match msg {
            Msg::ConnectWallet => {
                spawn_local({
                    let cb = ctx.props().context_cb.clone();

                    async move {
                        if let Some(context) = Web3Context::new().await {
                            set_wallet_addr(display_address(context.addr));

                            cb.emit((None, Some(context), None, None));
                        }
                    }
                });

                false
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        #[cfg(debug_assertions)]
        info!("Wallet Setting View");

        html! {
        <Section>
            <Container>
                <Subtitle >
                    {"Crypto Wallet"}
                </Subtitle>
                {
                    match self.address {
                        Some(addr) => self.render_connected(addr),
                        None => self.render_unconnected(),
                    }
                }
            </Container>
        </Section>
        }
    }
}

impl WalletSettings {
    fn render_connected(&self, addr: Address) -> Html {
        html! {
            <div class="field">
                <label class="label"> { "Wallet Address" } </label>
                <div class="control is-expanded">
                    <input name="wallet_addrs" value={display_address(addr)} class="input is-static" type="text" readonly=true />
                </div>
                <p class="help"> { "Wallet address used by this App." } </p>
            </div>
        }
    }

    fn render_unconnected(&self) -> Html {
        html! {
            <Button onclick={self.wallet_cb.clone()}>
                {"Connect Wallet"}
            </Button>
        }
    }
}
