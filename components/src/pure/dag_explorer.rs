#![cfg(target_arch = "wasm32")]

use cid::Cid;

use ybc::ButtonAnchor;

use yew::{classes, function_component, html, Html, Properties};

#[derive(Properties, PartialEq)]
pub struct DagExplorerProps {
    pub cid: Cid,
}

/// Turn a CID into a link to raw block data via IPFS web app.
#[function_component(DagExplorer)]
pub fn explore_dag(props: &DagExplorerProps) -> Html {
    let href = format!("https://webui.ipfs.io/#/explore/{}", props.cid.to_string());

    html! {
        <ButtonAnchor classes={classes!("is-small", "is-outlined", "is-primary", "is-rounded")} {href} >
            <small>{ "D.A.G. Explorer" }</small>
        </ButtonAnchor>
    }
}
