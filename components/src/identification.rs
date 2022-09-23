#![cfg(target_arch = "wasm32")]

use cid::Cid;

use gloo_console::error;

use utils::ipfs::IPFSContext;

use ybc::{ImageSize, Level, LevelItem, LevelRight};
use yew::{platform::spawn_local, prelude::*};

use linked_data::identity::Identity;
use yew_router::prelude::Link;

use crate::{image::Image, navbar::Route};

#[derive(Properties, PartialEq)]
pub struct Props {
    pub cid: Cid,

    pub addr: Option<String>,
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
                match ipfs.dag_get::<&str, Identity>(cid, None).await {
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

    fn view(&self, ctx: &Context<Self>) -> Html {
        match &self.identity {
            Some(identity) => {
                let img = match identity.avatar {
                    Some(avatar) => html! {
                        <ybc::Image size={ImageSize::Is64x64} >
                            <Image cid={avatar.link} round=true />
                        </ybc::Image>
                    },
                    None => html!(), //TODO default avatar
                };

                let check = match (identity.addr.as_ref(), ctx.props().addr.as_ref()) {
                    (Some(id_addr), Some(content_addr)) if content_addr == id_addr => {
                        html! {
                        <span class="icon">
                            <i class="fa-solid fa-check"></i>
                        </span>
                        }
                    }
                    _ => html!(),
                };

                let core = html! {
                    <Level>
                        <LevelRight>
                            <LevelItem>
                                {img}
                            </LevelItem>
                            <LevelItem>
                                <span class="icon-text">
                                    <span class="icon"><i class="fas fa-user"></i></span>
                                    <span> { &identity.display_name } </span>
                                    {check}
                                </span>
                            </LevelItem>
                        </LevelRight>
                    </Level>
                };

                if let Some(addr) = identity.channel_ipns {
                    return html! {
                        <Link<Route> to={Route::Channel{ addr: addr.into()}} >
                            {core}
                        </Link<Route>>
                    };
                }

                core
            }
            None => html! {
                <span class="bulma-loader-mixin"></span>
            },
        }
    }
}
