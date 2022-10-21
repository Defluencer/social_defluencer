#![cfg(target_arch = "wasm32")]

use cid::Cid;

use linked_data::types::IPNSAddress;

use utils::web3::Web3Context;

use yew::{
    function_component, html, platform::spawn_local, use_context, use_state, Callback, Html,
};

use yew_router::prelude::*;

use ybc::{Button, Input};

use crate::Route;

use gloo_console::error;

/* #[cfg(debug_assertions)]
use gloo_console::info; */

#[function_component(SearchBar)]
pub fn search_bar() -> Html {
    let context = match use_context::<Web3Context>() {
        Some(context) => context,
        None => return html! {},
    };

    let navigator = match use_navigator() {
        Some(nav) => nav,
        None => return html! {},
    };

    let text_state = use_state(|| String::new());
    let loading_state = use_state(|| false);

    let update = {
        let state = text_state.clone();

        Callback::from(move |text: String| state.set(text))
    };

    let ondone = {
        let loading = loading_state.clone();
        let text = text_state.clone();

        Callback::from(move |addr| {
            loading.set(false);
            text.set(String::new());

            if let Some(addr) = addr {
                navigator.push(&Route::Channel { addr });
            }
        })
    };

    let onclick = {
        let loading = loading_state.clone();
        let text = text_state.clone();

        Callback::from(move |_| {
            loading.set(true);

            spawn_local(get_channel(
                context.clone(),
                (*text).clone(),
                ondone.clone(),
            ));
        })
    };

    html! {
    <>
        <Input name="search_bar" placeholder={"Ethereum Name Service"} value={(*text_state).clone()} {update} />
        <Button {onclick} loading={*loading_state} >{"Search"}</Button>
    </>
    }
}

async fn get_channel(context: Web3Context, text: String, callback: Callback<Option<IPNSAddress>>) {
    /* if !text.starts_with("defluencer.") {
        text.insert_str(0, "defluencer.");
    }

    if !text.ends_with(".eth") {
        text.push_str(".eth");
    } */

    match context.ens.content_hash(text.as_str()).await {
        Ok(hash) => {
            match Cid::try_from(&hash[2..] /* First 2 bytes are protoCode uvarint */) {
                Ok(cid) => match IPNSAddress::try_from(cid) {
                    Ok(addr) => {
                        callback.emit(Some(addr));
                        return;
                    }
                    Err(e) => error!(&format!("{:#?}", e)),
                },
                Err(e) => error!(&format!("{:#?}", e)),
            }
        }
        Err(e) => error!(&format!("{:#?}", e)),
    }

    callback.emit(None);
}
