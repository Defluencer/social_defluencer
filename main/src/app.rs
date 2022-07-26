use yew::prelude::*;
use yew_router::prelude::*;

use channel::ChannelPage;
use content::ContentPage;
use feed::FeedPage;
use home::HomePage;
use live::LivePage;
use settings::SettingPage;

use cid::Cid;

//use wasm_bindgen_futures::spawn_local;

use utils::{ipfs::IPFSContext, web3::Web3Context};

use gloo_console::info;

//TODO fix hash router not working

#[derive(Routable, Clone, PartialEq)]
enum Route {
    #[at("/#/home")]
    Home, // social.defluencer.eth/#/home/

    #[at("/#/channel/:cid")]
    Channel { cid: Cid }, // social.defluencer.eth/#/channel/<IPNS_HERE>

    #[at("/#/content/:cid")]
    Content { cid: Cid }, // social.defluencer.eth/#/content/<CID_HERE>

    #[at("/#/feed")]
    Feed, // social.defluencer.eth/#/feed/

    #[at("/#/live/:cid")]
    Live { cid: Cid }, // social.defluencer.eth/#/live/<CID_HERE>

    #[at("/#/settings")]
    Settings,
}

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
        let web3_cb = ctx.link().callback(Msg::Web3);

        // init web3 and ipfs here?

        Self {
            ipfs_cb,
            ipfs_context: None,
            web3_cb,
            web3_context: None,
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        #[cfg(debug_assertions)]
        info!("App View");

        let ipfs_cb = self.ipfs_cb.clone();
        let web3_cb = self.web3_cb.clone();

        let app = html! {
        <BrowserRouter>
            <Switch<Route> render={Switch::render(move |route| {
                match route {
                    Route::Channel { cid } => html!{ <ChannelPage cid={*cid} /> },
                    Route::Content { cid } => html!{ <ContentPage cid={*cid} /> },
                    Route::Feed => html!{ <FeedPage /> },
                    Route::Home => html!{ <HomePage /> },
                    Route::Live { cid } => html!{ <LivePage cid={*cid} />},
                    Route::Settings => html!{ <SettingPage ipfs_cb={ipfs_cb.clone()} web3_cb={web3_cb.clone()} /> },
                }})}
             />
        </BrowserRouter>
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

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        #[cfg(debug_assertions)]
        info!("App Update");

        match msg {
            Msg::IPFS(context) => self.on_ipfs(context),
            Msg::Web3(context) => self.on_web3(context),
        }
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
