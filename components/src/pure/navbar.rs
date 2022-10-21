#![cfg(target_arch = "wasm32")]

use utils::defluencer::ChannelContext;

use yew::{classes, function_component, html, use_context, Html};

use yew_router::prelude::*;

use ybc::{Image, ImageSize, Level, LevelItem, LevelLeft, Navbar};

use crate::{search_bar::SearchBar, Route};

#[function_component(NavigationBar)]
pub fn navbar() -> Html {
    let channel_context = use_context::<ChannelContext>();

    let follow_list = utils::follows::get_follow_list();

    let navbrand = html! {
        <Link<Route> classes="navbar-item" to={Route::Home}>
            <Level>
                <LevelLeft>
                    <LevelItem>
                        <Image size={ImageSize::Is32x32} >
                            <img src="../image/defluencer_logo_blur.svg" alt="defluencer-logo" />
                        </Image>
                    </LevelItem>
                    <LevelItem>
                        <strong>{"Home"}</strong>
                    </LevelItem>
                </LevelLeft>
            </Level>
        </Link<Route>>
    };

    let navstart = html! {
        <>
            if !follow_list.is_empty(){
            <Link<Route> classes="navbar-item" to={Route::Feed}>
                <span class="icon-text">
                    <span class="icon"><i class="fas fa-broadcast-tower"></i></span>
                    <span><strong>{"Content Feed"}</strong></span>
                </span>
            </Link<Route>>
            }
            if let Some(context) = channel_context {
            <Link<Route> classes="navbar-item" to={Route::Channel { addr: context.channel.get_address() }}>
                <span class="icon-text">
                    <span class="icon"><i class="fa-solid fa-house-user"></i></span>
                    <span><strong>{"My Channel"}</strong></span>
                </span>
            </Link<Route>>
            }
            <SearchBar />
        </>
    };

    let navend = html! {
        <Link<Route> classes="navbar-item" to={Route::Settings}>
            <span class="icon-text" >
                <span class="icon"><i class="fas fa-cog"></i></span>
                <span><strong>{"Settings"}</strong></span>
            </span>
        </Link<Route>>
    };

    html! {
        <Navbar classes={classes!("is-spaced")} transparent=false spaced=true padded=false {navbrand} {navstart} {navend} navburger=true />
    }
}
