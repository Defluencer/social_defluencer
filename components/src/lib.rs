#![cfg(target_arch = "wasm32")]

use cid::Cid;

use linked_data::types::IPNSAddress;
use yew_router::Routable;

mod ema;
mod md_renderer;

pub mod chat;
pub mod comment_button;
pub mod markdown;
pub mod pure;
pub mod share_button;
pub mod video_player;

#[derive(Routable, Clone, PartialEq)]
pub enum Route {
    #[at("/channel/:addr")]
    Channel { addr: IPNSAddress }, // social.defluencer.eth/#/channel/<IPNS_HERE>

    #[at("/content/:cid")]
    Content { cid: Cid }, // social.defluencer.eth/#/content/<CID_HERE>

    #[at("/feed")]
    Feed, // social.defluencer.eth/#/feed/

    #[at("/live/:cid")]
    Live { cid: Cid }, // social.defluencer.eth/#/live/<CID_HERE>

    #[at("/settings")]
    Settings,

    #[not_found]
    #[at("/home")]
    Home, // social.defluencer.eth/#/home/
}
