#![cfg(target_arch = "wasm32")]

use cid::{multibase::Base, Cid};

use ipfs_api::IpfsService;

use utils::ipfs::IPFSContext;

use yew::{platform::spawn_local, prelude::*};

use linked_data::media::mime_type::MimeTyped;

use gloo_console::error;

//TODO use http://CID_HERE.ipfs.localhost://8080 instead of dag getting MimeTyped then the raw image bytes.

#[derive(Properties, PartialEq)]
pub struct Props {
    pub cid: Cid,

    pub round: bool,
}

pub struct Image {
    object_url: String,
}

pub enum Msg {
    Url(String),
}

impl Component for Image {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let (context, _handle) = ctx
            .link()
            .context::<IPFSContext>(Callback::noop())
            .expect("IPFS Context");

        spawn_local(get_image_url(
            context.client.clone(),
            ctx.props().cid,
            ctx.link().callback(Msg::Url),
        ));

        Self {
            object_url: String::new(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Url(url) => {
                self.object_url = url;

                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <img class={ if ctx.props().round { classes!("is-rounded") } else {classes!()} }src={self.object_url.clone()} />
        }
    }
}

async fn get_image_url(ipfs: IpfsService, cid: Cid, callback: Callback<String>) {
    let (img_res, data_res) = futures_util::join!(
        ipfs.dag_get::<&str, MimeTyped>(cid, None),
        ipfs.cat(cid, Some("/data"))
    );

    let image = match img_res {
        Ok(img) => img,
        Err(e) => {
            error!(&format!("{:#?}", e));
            return;
        }
    };

    let data = match data_res {
        Ok(bytes) => bytes,
        Err(e) => {
            error!(&format!("{:#?}", e));
            return;
        }
    };

    let url = format!(
        "data:{};base64,{}",
        image.mime_type,
        Base::Base64.encode(data)
    );

    callback.emit(url);
}
