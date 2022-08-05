#![cfg(target_arch = "wasm32")]

use cid::Cid;

use ybc::ButtonAnchor;

use yew::{classes, function_component, html, Html, Properties};

#[derive(Properties, PartialEq)]
pub struct CidExplorerProps {
    pub cid: Cid,
}

/// Turn a CID into a link to raw block data via IPFS web app.
#[function_component(CidExplorer)]
pub fn explore_cid(props: &CidExplorerProps) -> Html {
    let cid_string = props.cid.to_string();
    let href = format!("https://webui.ipfs.io/#/explore/{}", cid_string);

    html! {
        <ButtonAnchor classes={classes!("is-small", "is-outlined", "is-primary")} {href} >
            { cid_string }
        </ButtonAnchor>
    }
}
