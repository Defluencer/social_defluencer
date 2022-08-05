#![cfg(target_arch = "wasm32")]

// social.defluencer.eth/#/channel/<IPNS_HERE>
// Stream content metadata from channel
// Subscribe to the IPNS pubsub for live updates
// If your channel, add buttons to post & remove stuff
// If live, display video

use yew::prelude::*;

use cid::Cid;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub cid: Cid,
}

pub struct ChannelPage;

pub enum Msg {}

impl Component for ChannelPage {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! { "Channel Page" }
    }
}
