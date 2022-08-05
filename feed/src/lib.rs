#![cfg(target_arch = "wasm32")]

use yew::{function_component, html, Html};

use components::navbar::NavigationBar;

/// The Personal Feed Page.
#[function_component(FeedPage)]
pub fn feed() -> Html {
    html! {
        <>
        <NavigationBar />
        { "Feed page WIP" }
        </>
    }
}

// social.defluencer.eth/#/feed/
// Load channel list from storage
// Stream content metadata from all channel
