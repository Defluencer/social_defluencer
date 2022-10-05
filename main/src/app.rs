#![cfg(target_arch = "wasm32")]

use channel::ChannelPage;
use components::Route;
use content::ContentPage;
use feed::FeedPage;
use home::HomePage;
use live::LivePage;
use settings::SettingPage;

use linked_data::identity::Identity;

use yew::{platform::spawn_local, prelude::*};

use yew_router::prelude::*;

use utils::{
    defluencer::{ChannelContext, UserContext},
    identity::get_current_identity,
    ipfs::{get_ipfs_addr, set_ipfs_addr, IPFSContext},
    web3::{get_wallet_addr, Web3Context},
};

#[cfg(debug_assertions)]
use gloo_console::info;

pub enum Msg {
    Context(
        (
            Option<IPFSContext>,
            Option<Web3Context>,
            Option<UserContext>,
            Option<ChannelContext>,
        ),
    ),
}

pub struct App {
    context_cb: Callback<(
        Option<IPFSContext>,
        Option<Web3Context>,
        Option<UserContext>,
        Option<ChannelContext>,
    )>,

    ipfs_context: Option<IPFSContext>,
    web3_context: Option<Web3Context>,
    user_context: Option<UserContext>,
    channel_context: Option<ChannelContext>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        #[cfg(debug_assertions)]
        info!("App Create");

        let context_cb = ctx.link().callback(Msg::Context);

        spawn_local(get_context(context_cb.clone()));

        Self {
            context_cb,

            ipfs_context: None,
            web3_context: None,
            user_context: None,
            channel_context: None,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        #[cfg(debug_assertions)]
        info!("App Update");

        let mut update = false;

        match msg {
            Msg::Context((ipfs, web3, user, channel)) => {
                if self.ipfs_context.is_none() && ipfs.is_some() {
                    self.ipfs_context = ipfs;
                    update = true;
                }

                if self.web3_context.is_none() && web3.is_some() {
                    self.web3_context = web3;
                    update = true;
                }

                if self.user_context.is_none() && user.is_some() {
                    self.user_context = user;
                    update = true;
                }

                if self.channel_context.is_none() && channel.is_some() {
                    self.channel_context = channel;
                    update = true;
                }
            }
        }

        update
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        #[cfg(debug_assertions)]
        info!("App View");

        let context_cb = self.context_cb.clone();

        // If IPFS is not working displaying pages is pointless
        let app = match self.ipfs_context.as_ref() {
            Some(context) => html! {
                <ContextProvider<IPFSContext> context={context.clone()} >
                    <HashRouter>
                        <Switch<Route> render={move |route| {
                            match route {
                                Route::Channel { addr } => html!{ <ChannelPage {addr} /> },
                                Route::Content { cid } => html!{ <ContentPage {cid} /> },
                                Route::Feed => html!{ <FeedPage /> },
                                Route::Home => html!{ <HomePage /> },
                                Route::Live { cid } => html!{ <LivePage {cid} />},
                                Route::Settings => html!{ <SettingPage context_cb={context_cb.clone()} /> },
                            }}}
                        />
                    </HashRouter>
                </ContextProvider<IPFSContext>>
            },
            None => html! {
                <HashRouter>
                    <Switch<Route> render={move |route| {
                        match route {
                            Route::Home => html!{ <HomePage /> },
                            _ => html!{ <SettingPage context_cb={context_cb.clone()} /> },
                        }}}
                    />
                </HashRouter>
            },
        };

        let app = match self.web3_context.as_ref() {
            Some(context) => html! {
                <ContextProvider<Web3Context> context={context.clone()} >
                    {app}
                </ContextProvider<Web3Context>>
            },
            None => app,
        };

        let app = match self.user_context.as_ref() {
            Some(context) => html! {
                <ContextProvider<UserContext> context={context.clone()} >
                    {app}
                </ContextProvider<UserContext>>
            },
            None => app,
        };

        let app = match self.channel_context.as_ref() {
            Some(context) => html! {
                <ContextProvider<ChannelContext> context={context.clone()} >
                    {app}
                </ContextProvider<ChannelContext>>
            },
            None => app,
        };

        app
    }
}

async fn get_context(
    callback: Callback<(
        Option<IPFSContext>,
        Option<Web3Context>,
        Option<UserContext>,
        Option<ChannelContext>,
    )>,
) {
    let addr = get_ipfs_addr();

    let ipfs = IPFSContext::new(Some(&addr)).await;

    if ipfs.is_some() {
        set_ipfs_addr(&addr);
    }

    let mut web3 = None;

    if get_wallet_addr().is_some() {
        web3 = Web3Context::new().await;
    }

    let user = match (&ipfs, &web3, get_current_identity()) {
        (Some(ipfs), Some(web3), Some(ipld)) => {
            let context = UserContext::new(ipfs.client.clone(), web3.signer.clone(), ipld.link);

            Some(context)
        }
        _ => None,
    };

    let channel = match (&ipfs, get_current_identity()) {
        (Some(ipfs), Some(ipld)) => {
            match ipfs.client.dag_get::<&str, Identity>(ipld.link, None).await {
                Ok(identity) => {
                    if let Some(addr) = identity.ipns_addr {
                        use heck::ToSnakeCase;
                        let key = identity.name.to_snake_case();

                        let context = ChannelContext::new(ipfs.client.clone(), key, addr);

                        Some(context)
                    } else {
                        None
                    }
                }
                Err(_) => None,
            }
        }
        _ => None,
    };

    callback.emit((ipfs, web3, user, channel));
}
