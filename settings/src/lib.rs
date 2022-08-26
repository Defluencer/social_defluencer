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

use components::navbar::NavigationBar;

#[derive(Properties, PartialEq)]
pub struct SettingPageProps {
    pub ipfs_cb: Callback<IPFSContext>,
    pub web3_cb: Callback<Web3Context>,
    pub user_cb: Callback<UserContext>,
    pub channel_cb: Callback<ChannelContext>,
}

#[function_component(SettingPage)]
pub fn settings(props: &SettingPageProps) -> Html {
    let ipfs_cb = props.ipfs_cb.clone();
    let web3_cb = props.web3_cb.clone();

    let ipfs_context = use_context::<IPFSContext>();
    let web3_context = use_context::<Web3Context>();

    let identity_settings = match (ipfs_context, web3_context) {
        (Some(_), Some(_)) => {
            let user_cb = props.user_cb.clone();
            let channel_cb = props.channel_cb.clone();

            html! {<IdentitySettings {user_cb} {channel_cb} />}
        }
        _ => html! {},
    };

    html! {
        <>
        <NavigationBar />
        <IPFSSettings {ipfs_cb} />
        <WalletSettings {web3_cb} />
        {identity_settings}
        </>
    }
}
