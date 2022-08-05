#![cfg(target_arch = "wasm32")]

use yew::prelude::*;
use yew_router::prelude::*;

use channel::ChannelPage;
use content::ContentPage;
use feed::FeedPage;
use home::HomePage;
use live::LivePage;
use settings::SettingPage;

use components::navbar::Route;

use utils::{
    ipfs::{get_ipfs_addr, IPFSContext},
    web3::{get_wallet_addr, Web3Context},
};

use wasm_bindgen_futures::spawn_local;

use gloo_console::info;

pub enum Msg {
    IPFS(IPFSContext),
    Web3(Web3Context),
}

pub struct App {
    ipfs_cb: Callback<IPFSContext>,
    ipfs_context: Option<IPFSContext>,

    web3_cb: Callback<Web3Context>,
    web3_context: Option<Web3Context>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        #[cfg(debug_assertions)]
        info!("App Create");

        let ipfs_cb = ctx.link().callback(Msg::IPFS);

        if let Ok(url) = get_ipfs_addr() {
            spawn_local({
                let cb = ipfs_cb.clone();

                async move {
                    if let Some(context) = IPFSContext::new(Some(url)).await {
                        cb.emit(context);
                    }
                }
            });
        }

        let web3_cb = ctx.link().callback(Msg::Web3);

        if get_wallet_addr().is_some() {
            spawn_local({
                let cb = web3_cb.clone();

                async move {
                    if let Some(context) = Web3Context::new().await {
                        cb.emit(context);
                    }
                }
            });
        }

        Self {
            ipfs_cb,
            ipfs_context: None,
            web3_cb,
            web3_context: None,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        #[cfg(debug_assertions)]
        info!("App Update");

        match msg {
            Msg::IPFS(context) => self.on_ipfs(context),
            Msg::Web3(context) => self.on_web3(context),
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        #[cfg(debug_assertions)]
        info!("App View");

        let ipfs_cb = self.ipfs_cb.clone();
        let web3_cb = self.web3_cb.clone();

        let app = html! {
            <HashRouter>
                <Switch<Route> render={move |route| {
                    match route {
                        Route::Channel { cid } => html!{ <ChannelPage {cid} /> },
                        Route::Content { cid } => html!{ <ContentPage {cid} /> },
                        Route::Feed => html!{ <FeedPage /> },
                        Route::Home => html!{ <HomePage /> },
                        Route::Live { cid } => html!{ <LivePage {cid} />},
                        Route::Settings => html!{ <SettingPage ipfs_cb={ipfs_cb.clone()} web3_cb={web3_cb.clone()} /> },
                    }}}
                />
            </HashRouter>
        };

        let app = if let Some(context) = self.ipfs_context.as_ref() {
            html! {
            <ContextProvider<IPFSContext> context={context.clone()} >
                {app}
            </ContextProvider<IPFSContext>>
            }
        } else {
            app
        };

        let app = if let Some(context) = self.web3_context.as_ref() {
            html! {
            <ContextProvider<Web3Context> context={context.clone()} >
                {app}
            </ContextProvider<Web3Context>>
            }
        } else {
            app
        };

        app
    }
}

impl App {
    fn on_ipfs(&mut self, context: IPFSContext) -> bool {
        self.ipfs_context = Some(context);

        true
    }

    fn on_web3(&mut self, context: Web3Context) -> bool {
        self.web3_context = Some(context);

        true
    }
}
