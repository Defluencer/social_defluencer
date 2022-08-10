#![cfg(target_arch = "wasm32")]

use utils::ipfs::IPFSContext;

use wasm_bindgen_futures::spawn_local;

use linked_data::{
    comments::Comment,
    media::{
        blog::{FullPost, MicroPost},
        video::Video,
        Media,
    },
};

use ybc::{Container, Section};

use yew::prelude::*;

use cid::Cid;

use gloo_console::{error, info};

use components::loading::Loading;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub cid: Cid,
}

pub struct Content {
    media: Option<Media>,
    _handle: ContextHandle<IPFSContext>,
}

pub enum Msg {
    Media(Media),
}

impl Component for Content {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        #[cfg(debug_assertions)]
        info!("Content Page Create");

        let (context, _handle) = ctx
            .link()
            .context::<IPFSContext>(Callback::noop())
            .expect("IPFS Context");

        spawn_local({
            let cb = ctx.link().callback(Msg::Media);
            let ipfs = context.client.clone();
            let cid = ctx.props().cid;

            async move {
                match ipfs.dag_get::<String, Media>(cid, None).await {
                    Ok(dag) => cb.emit(dag),
                    Err(e) => error!(&format!("{:#?}", e)),
                }
            }
        });

        Self {
            media: None,
            _handle,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Media(media) => {
                self.media = Some(media);

                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        #[cfg(debug_assertions)]
        info!("Content Page View");

        html! {
        <Section>
            <Container>
            {
                match &self.media {
                    Some(media) => match media {
                        Media::MicroBlog(blog) => self.render_microblog(blog),
                        Media::Blog(blog) => self.render_article(blog),
                        Media::Video(video) => self.render_video(video),
                        Media::Comment(comment) => self.render_comment(comment),
                    },
                    None => self.render_loading(),
                }
            }
            </Container>
        </Section>
        }
    }
}

impl Content {
    fn render_loading(&self) -> Html {
        html! {
            <Loading />
        }
    }

    fn render_microblog(&self, blog: &MicroPost) -> Html {
        html! {
            <>

            </>
        }
    }

    fn render_article(&self, article: &FullPost) -> Html {
        html! {
            <>

            </>
        }
    }

    fn render_video(&self, video: &Video) -> Html {
        html! {
            <>

            </>
        }
    }

    fn render_comment(&self, comment: &Comment) -> Html {
        html! {
            <>

            </>
        }
    }
}
