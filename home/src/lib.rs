#![cfg(target_arch = "wasm32")]

use yew::{function_component, html, Html};

/// The App Landing Page.
#[function_component(HomePage)]
pub fn home() -> Html {
    html! { "Home Page" }
}

// social.defluencer.eth/#/home/
// Header This is decentralized social media
// Features explanations
// Button to start -> config IPFS
// Footer github, gitcoin, etc...
