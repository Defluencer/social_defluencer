#![cfg(target_arch = "wasm32")]

use linked_data::types::Address;

use ybc::{Block, Button, Container, Control, Field, Input, Section, Subtitle};

use yew::{platform::spawn_local, prelude::*};

use utils::{
    defluencer::{ChannelContext, UserContext},
    display_address,
    ipfs::IPFSContext,
    web3::{set_wallet_addr, Web3Context},
};

#[cfg(debug_assertions)]
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

        let address = match ctx.link().context::<Web3Context>(Callback::noop()) {
            Some((context, _)) => Some(context.addr),
            None => None,
        };

        let wallet_cb = ctx.link().callback(|_e: MouseEvent| Msg::ConnectWallet);

        Self { address, wallet_cb }
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
                            set_wallet_addr(&display_address(context.addr));

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
                <Subtitle>
                    {"Crypto Wallet"}
                </Subtitle>
                <Block>
                {"Metamask is required for now as there's no way to sign content with IPNS key or sign IPNS records with Metamask. I'm working on a better system, stay tuned!"}
                </Block>
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
        <Block>
            <Field label="Wallet Address" help={"Wallet address used by this App."}>
                <Control expanded=true >
                    <Input name="wallet_addrs" value={display_address(addr)} update={Callback::noop()} r#static=true readonly=true />
                </Control>
            </Field>
        </Block>
        }
    }

    fn render_unconnected(&self) -> Html {
        html! {
            <Button onclick={self.wallet_cb.clone()}>
                {"Connect"}
            </Button>
        }
    }
}
