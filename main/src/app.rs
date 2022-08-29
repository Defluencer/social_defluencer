#![cfg(target_arch = "wasm32")]

use linked_data::identity::Identity;
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
    defluencer::{ChannelContext, UserContext},
    identity::get_current_identity,
    ipfs::{get_ipfs_addr, IPFSContext},
    web3::{get_wallet_addr, Web3Context},
};

use wasm_bindgen_futures::spawn_local;

use gloo_console::{error, info};

pub enum Msg {
    IPFS(IPFSContext),
    Web3(Web3Context),
    User(UserContext),
    Channel(ChannelContext),
}

pub struct App {
    ipfs_cb: Callback<IPFSContext>,
    web3_cb: Callback<Web3Context>,
    user_cb: Callback<UserContext>,
    channel_cb: Callback<ChannelContext>,

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

        let ipfs_cb = ctx.link().callback(Msg::IPFS);

        // Get IPFS at startup from saved value
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

        // Get Web3 at startup from saved value
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

        let user_cb = ctx.link().callback(Msg::User);
        let channel_cb = ctx.link().callback(Msg::Channel);

        Self {
            ipfs_cb,
            web3_cb,
            user_cb,
            channel_cb,

            ipfs_context: None,
            web3_context: None,
            user_context: None,
            channel_context: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        #[cfg(debug_assertions)]
        info!("App Update");

        match msg {
            Msg::IPFS(context) => self.ipfs_context = Some(context),
            Msg::Web3(context) => self.web3_context = Some(context),
            Msg::User(context) => self.user_context = Some(context),
            Msg::Channel(context) => self.channel_context = Some(context),
        }

        // Get user at startup from saved value
        match (
            self.ipfs_context.as_ref(),
            self.web3_context.as_ref(),
            get_current_identity(),
            self.user_context.as_ref(),
        ) {
            (Some(ipfs), Some(web3), Some(ipld), None) => {
                let context = UserContext::new(ipfs.client.clone(), web3.signer.clone(), ipld.link);

                self.user_context = Some(context);
            }
            _ => {}
        }

        // Get Channel at startup from saved value
        match (
            self.ipfs_context.as_ref(),
            get_current_identity(),
            self.channel_context.as_ref(),
        ) {
            (Some(ipfs), Some(ipld), None) => spawn_local({
                let ipfs = ipfs.client.clone();
                let cb = ctx.link().callback(Msg::Channel);

                async move {
                    match ipfs.dag_get::<&str, Identity>(ipld.link, None).await {
                        Ok(identity) => {
                            if let Some(addr) = identity.channel_ipns {
                                use heck::ToSnakeCase;
                                let key = identity.display_name.to_snake_case();

                                let context = ChannelContext::new(ipfs.clone(), key, addr);

                                cb.emit(context)
                            }
                        }
                        Err(e) => error!(&format!("{:?}", e)),
                    }
                }
            }),
            _ => {}
        }

        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        #[cfg(debug_assertions)]
        info!("App View");

        let ipfs_cb = self.ipfs_cb.clone();
        let web3_cb = self.web3_cb.clone();
        let user_cb = self.user_cb.clone();
        let channel_cb = self.channel_cb.clone();

        // If IPFS is not working displaying pages is pointless
        let app = match self.ipfs_context.as_ref() {
            Some(context) => html! {
                <ContextProvider<IPFSContext> context={context.clone()} >
                    <HashRouter>
                        <Switch<Route> render={move |route| {
                            match route {
                                Route::Channel { cid } => html!{ <ChannelPage {cid} /> },
                                Route::Content { cid } => html!{ <ContentPage {cid} /> },
                                Route::Feed => html!{ <FeedPage /> },
                                Route::Home => html!{ <HomePage /> },
                                Route::Live { cid } => html!{ <LivePage {cid} />},
                                Route::Settings => html!{ <SettingPage ipfs_cb={ipfs_cb.clone()} web3_cb={web3_cb.clone()} user_cb={user_cb.clone()} channel_cb={channel_cb.clone()} /> },
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
                            _ => html!{ <SettingPage ipfs_cb={ipfs_cb.clone()} web3_cb={web3_cb.clone()} user_cb={user_cb.clone()} channel_cb={channel_cb.clone()} /> },
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
