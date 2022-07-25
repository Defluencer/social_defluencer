// social.defluencer.eth/#/content/<CID_HERE>
// Load the content CID
// Display the content according to type.
// Agregate comments while the user consume the content.
// Display comments

// Exporting content as CARs?
// Explore DAG?

use yew::prelude::*;

use cid::Cid;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub cid: Cid,
}

pub struct ContentPage;

pub enum Msg {}

impl Component for ContentPage {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! { "Content Page" }
    }
}
