// social.defluencer.eth/#/settings/
// User & channels section for switching identity or publishing channel.
// IPFS section, changes when connected

use yew::prelude::*;

use utils::{ipfs::IPFSContext, web3::Web3Context};

#[derive(Properties, PartialEq)]
pub struct Props {
    pub ipfs_cb: Callback<IPFSContext>,
    pub web3_cb: Callback<Web3Context>,
}

pub struct SettingPage;

pub enum Msg {}

impl Component for SettingPage {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {"Setting Page"}
    }
}
