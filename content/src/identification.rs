#![cfg(target_arch = "wasm32")]

use cid::Cid;

use gloo_console::error;
use utils::ipfs::IPFSContext;
use wasm_bindgen_futures::spawn_local;
use ybc::Block;
use yew::prelude::*;

use linked_data::identity::Identity;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub cid: Cid,
}

pub struct Identification {
    identity: Option<Identity>,
}

pub enum Msg {
    Identity(Identity),
}

impl Component for Identification {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let (context, _) = ctx
            .link()
            .context::<IPFSContext>(Callback::noop())
            .expect("IPFS Context");

        spawn_local({
            let cb = ctx.link().callback(Msg::Identity);
            let ipfs = context.client.clone();
            let cid = ctx.props().cid;

            async move {
                match ipfs.dag_get::<String, Identity>(cid, None).await {
                    Ok(id) => cb.emit(id),
                    Err(e) => error!(&format!("{:#?}", e)),
                }
            }
        });

        Self { identity: None }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Identity(id) => {
                self.identity = Some(id);

                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        match &self.identity {
            Some(identity) => html! {
                <Block>
                    <span class="icon-text">
                        <span class="icon"><i class="fas fa-user"></i></span>
                        <span> { &identity.display_name } </span>
                    </span>
                </Block>
            },
            None => html! {
                <span class="bulma-loader-mixin"></span>
            },
        }
    }
}
