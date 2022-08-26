#![cfg(target_arch = "wasm32")]

use cid::{multibase::Base, Cid};

use either::Either;

use ipfs_api::IpfsService;

use utils::ipfs::IPFSContext;

use yew::{platform::spawn_local, prelude::*};

use linked_data::media::mime_type::MimeTyped;

use gloo_console::error;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub cid: Cid,
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

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <img src={self.object_url.clone()} />
        }
    }
}

async fn get_image_url(ipfs: IpfsService, cid: Cid, callback: Callback<String>) {
    let image = match ipfs.dag_get::<&str, MimeTyped>(cid, None).await {
        Ok(img) => img,
        Err(e) => {
            error!(&format!("{:#?}", e));
            return;
        }
    };

    let data = match image.data {
        Either::Left(ipld) => {
            let bytes = match ipfs.cat(ipld.link, Option::<&str>::None).await {
                Ok(bytes) => bytes,
                Err(e) => {
                    error!(&format!("{:#?}", e));
                    return;
                }
            };

            bytes.to_vec()
        }
        Either::Right(vec) => vec,
    };

    let url = format!(
        "data:{};base64,{}",
        image.mime_type,
        Base::Base64.encode(data)
    );

    callback.emit(url);
}
