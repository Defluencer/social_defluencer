#![cfg(target_arch = "wasm32")]

use components::pure::NavigationBar;

use utils::defluencer::{ChannelContext, UserContext};
use utils::web3::Web3Context;
use yew::{html, Html};

use yew::prelude::*;

use gloo_console::info;

use utils::ipfs::IPFSContext;

#[derive(Properties, PartialEq)]
pub struct Props {}

/// social.defluencer.eth/#/home/
/// The App Landing Page.
pub struct HomePage {
    _ipfs_handle: Option<ContextHandle<IPFSContext>>,
    _user_handle: Option<ContextHandle<UserContext>>,
    _channel_handle: Option<ContextHandle<ChannelContext>>,
    _web3_handle: Option<ContextHandle<Web3Context>>,
}

pub enum Msg {
    IPFSContextUpdate(IPFSContext),
    UserContextUpdate(UserContext),
    ChannelContextUpdate(ChannelContext),
    Web3ContextUpdate(Web3Context),
}

// Header This is decentralized social media
// Features explanations
// Button to start -> config IPFS
// Footer github, gitcoin, etc...

//TODO remove experiment

impl Component for HomePage {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        info!("Home Page Create");

        let cb = ctx.link().callback(Msg::IPFSContextUpdate);
        let ipfs_handle = if let Some((_, handle)) = ctx.link().context::<IPFSContext>(cb) {
            info!("IPFS Context");

            Some(handle)
        } else {
            None
        };

        let cb = ctx.link().callback(Msg::UserContextUpdate);
        let user_handle = if let Some((_, handle)) = ctx.link().context::<UserContext>(cb) {
            info!("User Context");

            Some(handle)
        } else {
            None
        };

        let cb = ctx.link().callback(Msg::ChannelContextUpdate);
        let channel_handle = if let Some((_, handle)) = ctx.link().context::<ChannelContext>(cb) {
            info!("Channel Context");

            Some(handle)
        } else {
            None
        };

        let cb = ctx.link().callback(Msg::Web3ContextUpdate);
        let web3_handle = if let Some((_, handle)) = ctx.link().context::<Web3Context>(cb) {
            info!("Web3 Context");

            Some(handle)
        } else {
            None
        };

        Self {
            _ipfs_handle: ipfs_handle,
            _user_handle: user_handle,
            _channel_handle: channel_handle,
            _web3_handle: web3_handle,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        info!("Home Page Update");

        match msg {
            Msg::IPFSContextUpdate(_) => {
                info!("IPFS Context Update")
            }
            Msg::UserContextUpdate(_) => {
                info!("User Context Update")
            }
            Msg::ChannelContextUpdate(_) => {
                info!("Channel Context Update")
            }
            Msg::Web3ContextUpdate(_) => {
                info!("Web3 Context Update")
            }
        }

        false
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        info!("Home Page View");

        html! {
        <>
            <NavigationBar />
            { "Home Page WIP" }
        </>
        }
    }
}
