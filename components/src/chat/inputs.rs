#![cfg(target_arch = "wasm32")]

use cid::Cid;

use gloo_console::error;

use linked_data::live::LiveSettings;

use utils::ipfs::IPFSContext;

use yew::{platform::spawn_local, prelude::*};

pub struct ChatInputs {}

pub enum Msg {}

impl Component for ChatInputs {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <>

            </>
        }
    }
}

impl ChatInputs {}
