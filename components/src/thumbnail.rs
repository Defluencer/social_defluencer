#![cfg(target_arch = "wasm32")]

use cid::Cid;

use gloo_console::error;

use linked_data::media::{
    blog::{FullPost, MicroPost},
    video::Video,
    Media,
};

use utils::{ipfs::IPFSContext, seconds_to_timecode, timestamp_to_datetime};

use ybc::{
    Block, Box, HeaderSize, ImageSize, MediaContent, MediaLeft, MediaRight, Subtitle, Title,
};

use yew::{platform::spawn_local, prelude::*};

use yew_router::prelude::Link;

use crate::{
    cid_explorer::CidExplorer, identification::Identification, image::Image, navbar::Route,
};

#[derive(Properties, PartialEq)]
pub struct Props {
    /// Signed link to Media Content Cid
    pub cid: Cid,
}

pub struct Thumbnail {
    media: Option<Media>,
}

pub enum Msg {
    Metadata(Media),
}

impl Component for Thumbnail {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let (context, _) = ctx
            .link()
            .context::<IPFSContext>(Callback::noop())
            .expect("IPFS Context");

        spawn_local({
            let cb = ctx.link().callback(Msg::Metadata);
            let ipfs = context.client.clone();
            let cid = ctx.props().cid;

            async move {
                match ipfs.dag_get::<&str, Media>(cid, Some("/link")).await {
                    Ok(id) => cb.emit(id),
                    Err(e) => error!(&format!("{:#?}", e)),
                }
            }
        });

        Self { media: None }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Metadata(id) => {
                self.media = Some(id);

                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        match &self.media {
            Some(media) => {
                let dt = timestamp_to_datetime(media.user_timestamp());

                match media {
                    Media::Video(metadata) => self.render_video(ctx, dt, metadata),
                    Media::Blog(metadata) => self.render_blog(ctx, dt, metadata),
                    Media::MicroBlog(metadata) => self.render_microblog(ctx, dt, metadata),
                    _ => html! {},
                }
            }
            None => html! {
                <span class="bulma-loader-mixin"></span>
            },
        }
    }
}
impl Thumbnail {
    fn render_video(&self, ctx: &Context<Self>, dt: String, metadata: &Video) -> Html {
        let (hour, minute, second) = seconds_to_timecode(metadata.duration);

        html! {
        <Box>
            <ybc::Media>
                <MediaLeft>
                    <Block>
                        <Identification cid={metadata.identity.link} />
                    </Block>
                    <Block>
                        <span class="icon-text">
                            <span class="icon"><i class="fas fa-clock"></i></span>
                            <span> { dt } </span>
                        </span>
                    </Block>
                    <Block>
                        <span class="icon-text">
                            <span class="icon"><i class="fas fa-video"></i></span>
                            <span> { &format!("{}:{}:{}", hour, minute, second) } </span>
                        </span>
                    </Block>
                </MediaLeft>
                <MediaContent classes={classes!("has-text-centered")} >
                    <Link<Route> to={Route::Content{ cid: ctx.props().cid}} >
                        <Title classes={classes!("is-6")} >
                            { &metadata.title }
                        </Title>
                        <ybc::Image size={ImageSize::Is16by9} >
                            <Image cid={metadata.image.link} />
                        </ybc::Image>
                    </Link<Route>>
                </MediaContent>
                <MediaRight>
                    <CidExplorer cid={ctx.props().cid} />
                </MediaRight>
            </ybc::Media>
        </Box>
        }
    }

    fn render_blog(&self, ctx: &Context<Self>, dt: String, metadata: &FullPost) -> Html {
        html! {
        <Box>
            <ybc::Media>
                <MediaLeft>
                    <Block>
                        <Identification cid={metadata.identity.link} />
                    </Block>
                    <Block>
                        <span class="icon-text">
                            <span class="icon"><i class="fas fa-clock"></i></span>
                            <span> { dt } </span>
                        </span>
                    </Block>
                </MediaLeft>
                <MediaContent classes={classes!("has-text-centered")} >
                    <Link<Route> to={Route::Content{ cid: ctx.props().cid}} >
                        <Title classes={classes!("is-6")} >
                            { &metadata.title }
                        </Title>
                        <ybc::Image size={ImageSize::Is16by9} >
                            <Image cid={metadata.image.link} />
                        </ybc::Image>
                    </Link<Route>>
                </MediaContent>
                <MediaRight>
                    <CidExplorer cid={ctx.props().cid} />
                </MediaRight>
            </ybc::Media>
        </Box>
        }
    }

    fn render_microblog(&self, ctx: &Context<Self>, dt: String, metadata: &MicroPost) -> Html {
        html! {
        <Box>
            <ybc::Media>
                <MediaLeft>
                    <Block>
                        <Identification cid={metadata.identity.link} />
                    </Block>
                    <Block>
                        <span class="icon-text">
                            <span class="icon"><i class="fas fa-clock"></i></span>
                            <span> { dt } </span>
                        </span>
                    </Block>
                </MediaLeft>
                <MediaContent>
                    <Link<Route> to={Route::Content{ cid: ctx.props().cid}} >
                        <Subtitle classes={classes!("has-text-centered")} size={HeaderSize::Is6} >
                            { &metadata.content }
                        </Subtitle>
                    </Link<Route>>
                </MediaContent>
                <MediaRight>
                    <CidExplorer cid={ctx.props().cid} />
                </MediaRight>
            </ybc::Media>
        </Box>
        }
    }
}
