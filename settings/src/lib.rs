mod identity;
mod ipfs;
mod wallet;

use utils::{ipfs::IPFSContext, web3::Web3Context};

use yew::{function_component, html, Callback, Html, Properties};

use ipfs::IPFSSettings;

use wallet::WalletSettings;

use identity::IdentitySettings;

#[derive(Properties, PartialEq)]
pub struct SettingPageProps {
    pub ipfs_cb: Callback<IPFSContext>,
    pub web3_cb: Callback<Web3Context>,
}

#[function_component(SettingPage)]
pub fn settings(props: &SettingPageProps) -> Html {
    let ipfs_cb = props.ipfs_cb.clone();
    let web3_cb = props.web3_cb.clone();

    html! {
        <>
        <IPFSSettings {ipfs_cb} />
        <WalletSettings {web3_cb} />
        <IdentitySettings />
        </>
    }
}
