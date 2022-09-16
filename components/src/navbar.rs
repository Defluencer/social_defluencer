#![cfg(target_arch = "wasm32")]

use cid::Cid;

use yew::{classes, function_component, html, Html};

use yew_router::prelude::*;

use ybc::Navbar;

#[function_component(NavigationBar)]
pub fn navbar() -> Html {
    let navbrand = html! {
        <Link<Route> classes="navbar-item" to={Route::Home}>
            <img src="../image/defluencer_logo_blur.svg" alt="defluencer-logo" />
            {"Home"}
        </Link<Route>>
    };

    let navstart = html! {
        <>
            /* <Link<Route> classes="navbar-item" to={Route::Home}>
                <span class="icon-text">
                    <span class="icon"><i class="fas fa-directions"></i></span>
                    <span> {"Get Started"} </span>
                </span>
            </Link<Route>> */
            <Link<Route> classes="navbar-item" to={Route::Feed}>
                <span class="icon-text">
                    <span class="icon"><i class="fas fa-rss"></i></span>
                    <span> {"Content Feed"} </span>
                </span>
            </Link<Route>>
            /* <Link<Route> classes="navbar-item" to={Route::Live}>
                <span class="icon-text">
                    <span class="icon"><i class="fas fa-broadcast-tower"></i></span>
                    <span> {"Live"} </span>
                </span>
            </Link<Route>> */
        </>
    };

    let navend = html! {
        <Link<Route> classes="navbar-item" to={Route::Settings}>
            <span class="icon-text" >
                <span class="icon"><i class="fas fa-cog"></i></span>
                <span> {"Settings"} </span>
            </span>
        </Link<Route>>
    };

    html! {
        <Navbar classes={classes!("is-spaced")} transparent=false spaced=true padded=false {navbrand} {navstart} {navend} navburger=true />
    }
}

#[derive(Routable, Clone, PartialEq)]
pub enum Route {
    #[at("/channel/:addr")]
    Channel { addr: Cid }, // social.defluencer.eth/#/channel/<IPNS_HERE>

    #[at("/content/:cid")]
    Content { cid: Cid }, // social.defluencer.eth/#/content/<CID_HERE>

    #[at("/feed")]
    Feed, // social.defluencer.eth/#/feed/

    #[at("/live/:cid")]
    Live { cid: Cid }, // social.defluencer.eth/#/live/<CID_HERE>

    #[at("/settings")]
    Settings,

    #[not_found]
    #[at("/home")]
    Home, // social.defluencer.eth/#/home/
}
