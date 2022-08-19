#![cfg(target_arch = "wasm32")]

use yew::{function_component, html, Html};

use components::navbar::NavigationBar;

//? social.defluencer.eth/#/home/
/// The App Landing Page.
#[function_component(HomePage)]
pub fn home() -> Html {
    html! {
        <>
        <NavigationBar />
        { "Home Page WIP" }
        </>
    }
}

// Header This is decentralized social media
// Features explanations
// Button to start -> config IPFS
// Footer github, gitcoin, etc...
