#![cfg(target_arch = "wasm32")]

use cid::Cid;

use gloo_console::error;

use linked_data::live::LiveSettings;

use utils::ipfs::IPFSContext;

use yew::{platform::spawn_local, prelude::*};

use super::{display::ChatDisplay, inputs::ChatInputs};

#[derive(Properties, PartialEq)]
pub struct Props {
    pub cid: Cid,
}

pub struct ChatWindow {
    context: Option<LiveContext>,
}

pub enum Msg {
    Settings(LiveSettings),
}

impl Component for ChatWindow {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let (context, _) = ctx
            .link()
            .context::<IPFSContext>(Callback::noop())
            .expect("IPFS Context");

        spawn_local({
            let cb = ctx.link().callback(Msg::Settings);
            let ipfs = context.client.clone();
            let cid = ctx.props().cid;

            async move {
                match ipfs.dag_get::<&str, LiveSettings>(cid, None).await {
                    Ok(id) => cb.emit(id),
                    Err(e) => error!(&format!("{:#?}", e)),
                }
            }
        });

        Self { context: None }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Settings(settings) => {
                //TODO if chat is disabled display a msg

                self.context = Some(LiveContext { settings });

                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        if let Some(context) = self.context.as_ref() {
            html! {
            <ContextProvider<LiveContext> context={context.clone()} >
                <ChatDisplay />
                <ChatInputs />
            </ContextProvider<LiveContext>>
            }
        } else {
            html! { <span class="bulma-loader-mixin"></span> }
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct LiveContext {
    pub settings: LiveSettings,
}
