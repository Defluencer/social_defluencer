#![cfg(target_arch = "wasm32")]

mod identity;
mod ipfs;
mod wallet;

use utils::{
    defluencer::{ChannelContext, UserContext},
    ipfs::IPFSContext,
    web3::Web3Context,
};

use yew::{function_component, html, use_context, Callback, Html, Properties};

use ipfs::IPFSSettings;

use wallet::WalletSettings;

use identity::IdentitySettings;

use components::pure::NavigationBar;

#[derive(Properties, PartialEq)]
pub struct SettingPageProps {
    pub context_cb: Callback<(
        Option<IPFSContext>,
        Option<Web3Context>,
        Option<UserContext>,
        Option<ChannelContext>,
    )>,
}

#[function_component(SettingPage)]
pub fn settings(props: &SettingPageProps) -> Html {
    let context_cb = props.context_cb.clone();

    let ipfs_context = use_context::<IPFSContext>();
    let web3_context = use_context::<Web3Context>();

    let identity_settings = match (ipfs_context, web3_context) {
        (Some(_), Some(_)) => {
            html! {<IdentitySettings context_cb={context_cb.clone()} />}
        }
        _ => html! {},
    };

    html! {
        <>
        <NavigationBar />
        <IPFSSettings context_cb={context_cb.clone()} />
        <WalletSettings {context_cb} />
        {identity_settings}
        </>
    }
}
