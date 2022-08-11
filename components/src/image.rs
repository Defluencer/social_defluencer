#![cfg(target_arch = "wasm32")]

use cid::Cid;

use either::Either;

use ipfs_api::IpfsService;

use js_sys::Uint8Array;

use utils::ipfs::IPFSContext;

use yew::{platform::spawn_local, prelude::*};

use linked_data::media::mime_type::MimeTyped;

use web_sys::{File, FilePropertyBag, Url};

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

    fn destroy(&mut self, _ctx: &Context<Self>) {
        if let Err(e) = Url::revoke_object_url(&self.object_url) {
            error!(&format!("{:#?}", e))
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

    let file_bits: Uint8Array = data[..].into();

    let mut options = FilePropertyBag::new();
    options.type_(&image.mime_type);

    let file = match File::new_with_u8_array_sequence_and_options(
        &file_bits,
        &cid.to_string(),
        &options,
    ) {
        Ok(file) => file,
        Err(e) => {
            error!(&format!("{:#?}", e));
            return;
        }
    };

    let url = match Url::create_object_url_with_blob(&file) {
        Ok(url) => url,
        Err(e) => {
            error!(&format!("{:#?}", e));
            return;
        }
    };

    callback.emit(url);
}
