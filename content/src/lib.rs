#![cfg(target_arch = "wasm32")]

mod comment;
mod commentary;
mod content;
mod markdown;
mod md_renderer;

use commentary::Commentary;

use components::navbar::NavigationBar;

use content::Content;

use yew::{function_component, html, Html, Properties};

use cid::Cid;

#[derive(Properties, PartialEq)]
pub struct ContentPageProps {
    /// Signed Link to Media Cid
    pub cid: Cid,
}

/// social.defluencer.eth/#/content/<CID_HERE>
///
/// Page displaying specific content & comments
#[function_component(ContentPage)]
pub fn content_page(props: &ContentPageProps) -> Html {
    let cid = props.cid;

    html! {
    <>
    <NavigationBar />
    <Content {cid} />
    <Commentary {cid} />
    </>
    }
}
