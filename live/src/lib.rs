// social.defluencer.eth/#/live/<CID_HERE>
// Load live settings from CID
// Subscribe to video & chat
// Display video and chat

use yew::prelude::*;

use cid::Cid;

pub enum Msg {}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub cid: Cid,
}

pub struct LivePage;

impl Component for LivePage {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! { "Live Page" }
    }
}
