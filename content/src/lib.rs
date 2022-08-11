#![cfg(target_arch = "wasm32")]

mod comment;
mod commentary;
mod content;
mod identification;

use commentary::Commentary;

use components::navbar::NavigationBar;

use content::Content;

use yew::{function_component, html, Html, Properties};

use cid::Cid;

// social.defluencer.eth/#/content/<CID_HERE>
// Load the content CID
// Display the content according to type.
// Agregate comments while the user consume the content.
// Display comments

// Exporting content as CARs?
// Explore DAG?

#[derive(Properties, PartialEq)]
pub struct ContentPageProps {
    pub cid: Cid,
}

#[function_component(ContentPage)]
pub fn content_page(props: &ContentPageProps) -> Html {
    let cid = props.cid;

    //TODO check connection to ipfs

    //TODO if not connected display error

    html! {
    <>
    <NavigationBar />
    <Content {cid} />
    <Commentary {cid} />
    </>
    }
}
