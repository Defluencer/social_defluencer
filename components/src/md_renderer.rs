#![cfg(target_arch = "wasm32")]

use cid::Cid;

use gloo_console::error;

use ipfs_api::IpfsService;

use utils::ipfs::IPFSContext;

use yew::{platform::spawn_local, prelude::*};

use crate::markdown::render_markdown;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub cid: Cid,
}

pub struct Markdown {
    markdown: Html,
}

pub enum Msg {
    Text(Html),
}

impl Component for Markdown {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        if let Some((context, _)) = ctx.link().context::<IPFSContext>(Callback::noop()) {
            spawn_local(get_markdown_file(
                context.client,
                ctx.link().callback(Msg::Text),
                ctx.props().cid,
            ));
        }

        Self { markdown: html!() }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Text(html) => {
                self.markdown = html;

                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        self.markdown.clone()
    }
}

async fn get_markdown_file(ipfs: IpfsService, callback: Callback<Html>, cid: Cid) {
    let data = match ipfs.cat(cid, Option::<&str>::None).await {
        Ok(data) => data,
        Err(e) => {
            error!(&format!("{:#?}", e));
            return;
        }
    };

    let str = match std::str::from_utf8(&data) {
        Ok(str) => str,
        Err(e) => {
            error!(&format!("{:#?}", e));
            return;
        }
    };

    let html = render_markdown(str);

    callback.emit(html);
}
