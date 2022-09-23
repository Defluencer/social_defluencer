#![cfg(target_arch = "wasm32")]

use cid::Cid;

use yew::prelude::*;

use ybc::{Image, ImageSize};

#[derive(Properties, PartialEq)]
pub struct IPFSImageProps {
    pub cid: Cid,

    pub rounded: bool,

    pub size: ImageSize,
}

#[function_component(IPFSImage)]
pub fn pure_image(props: &IPFSImageProps) -> Html {
    let cid = props.cid;
    let size = props.size.clone();
    let rounded = props.rounded;

    html! {
    <Image {size} >
        <img class={ if rounded { "is-rounded"} else {""} } src={ format!("http://{}.ipfs.localhost:8080", cid) } />
    </Image>
    }
}
