#![cfg(target_arch = "wasm32")]

use yew::{classes, function_component, html, Html};

#[function_component(Loading)]
pub fn loading() -> Html {
    html! {
        <ybc::Container classes={classes!("has-text-centered")} >
            <ybc::Box>
                <div>
                    { "Searching the decentralized web. Please wait..." }
                </div>
                <progress class="progress is-primary is-small">
                    { "0%" }
                </progress>
            </ybc::Box>
        </ybc::Container>
    }
}
