#![cfg(target_arch = "wasm32")]

use defluencer::signatures::signed_link::SignedLink;

use ipfs_api::IpfsService;

use utils::{ipfs::IPFSContext, timestamp_to_datetime};

use linked_data::media::{
    blog::{FullPost, MicroPost},
    video::Video,
    Media,
};

use ybc::{
    Block, Box, Container, ImageSize, Level, LevelItem, LevelLeft, LevelRight, MediaContent,
    MediaLeft, MediaRight, Section, Title,
};

use yew::{platform::spawn_local, prelude::*};

use cid::Cid;

use gloo_console::{error, info};

use components::{
    cid_explorer::CidExplorer, image::Image, loading::Loading, video_player::VideoPlayer,
};

use crate::{comment::Comment, identification::Identification, md_renderer::Markdown};

#[derive(Properties, PartialEq)]
pub struct Props {
    pub cid: Cid,
}

pub struct Content {
    media: Option<Media>,
    dt: String,
    pk: Vec<u8>,
}

pub enum Msg {
    Media((Media, Vec<u8>)),
}

impl Component for Content {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        #[cfg(debug_assertions)]
        info!("Content Create");

        let (context, _) = ctx
            .link()
            .context::<IPFSContext>(Callback::noop())
            .expect("IPFS Context");

        spawn_local(get_content(
            context.client.clone(),
            ctx.link().callback(Msg::Media),
            ctx.props().cid,
        ));

        Self {
            media: None,
            dt: String::new(),
            pk: Vec::new(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        #[cfg(debug_assertions)]
        info!("Content Update");

        match msg {
            Msg::Media((media, pk)) => {
                self.dt = timestamp_to_datetime(media.user_timestamp());
                self.media = Some(media);
                self.pk = pk;

                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        #[cfg(debug_assertions)]
        info!("Content View");

        html! {
        <Section>
            <Container>
            {
                match &self.media {
                    Some(media) => match media {
                        Media::MicroBlog(blog) => self.render_microblog(ctx, blog),
                        Media::Blog(blog) => self.render_article(ctx, blog),
                        Media::Video(video) => self.render_video(ctx, video),
                        Media::Comment(_) => self.render_comment(ctx),
                    },
                    None => html! {
                            <Loading />
                        },
                }
            }
            </Container>
        </Section>
        }
    }
}

impl Content {
    fn render_microblog(&self, ctx: &Context<Self>, blog: &MicroPost) -> Html {
        html! {
        <Box>
            <ybc::Media>
                <MediaLeft>
                    <Identification cid={blog.identity.link} pk={Some(self.pk.clone())} />
                    <Block>
                        <span class="icon-text">
                            <span class="icon"><i class="fas fa-clock"></i></span>
                            <span> { &self.dt } </span>
                        </span>
                    </Block>
                </MediaLeft>
                <MediaContent>
                    <ybc::Content classes={classes!("has-text-centered")} >
                        { &blog.content }
                    </ybc::Content>
                </MediaContent>
                <MediaRight>
                    <CidExplorer cid={ctx.props().cid} />
                </MediaRight>
            </ybc::Media>
        </Box>
        }
    }

    fn render_article(&self, ctx: &Context<Self>, article: &FullPost) -> Html {
        html! {
        <Box>
            <Title>
                { &article.title }
            </Title>
            <ybc::Image size={ImageSize::Is16by9} >
                <Image cid={article.image.link}  />
            </ybc::Image>
            <Level>
                <LevelLeft>
                    <LevelItem>
                        <Identification cid={article.identity.link} pk={Some(self.pk.clone())} />
                    </LevelItem>
                    <LevelItem>
                        <span class="icon-text">
                            <span class="icon"><i class="fas fa-clock"></i></span>
                            <span> { &self.dt } </span>
                        </span>
                    </LevelItem>
                </LevelLeft>
                <LevelRight>
                    <LevelItem>
                        <CidExplorer cid={ctx.props().cid} />
                    </LevelItem>
                </LevelRight>
            </Level>
            <ybc::Content>
                <Markdown cid={article.content.link} />
            </ybc::Content>
        </Box>
        }
    }

    fn render_video(&self, ctx: &Context<Self>, video: &Video) -> Html {
        html! {
        <Box>
            <Title>
                { &video.title }
            </Title>
                <VideoPlayer cid={ctx.props().cid} />
            <Level>
                <LevelLeft>
                    <LevelItem>
                        <Identification cid={video.identity.link} pk={Some(self.pk.clone())} />
                    </LevelItem>
                    <LevelItem>
                        <span class="icon-text">
                            <span class="icon"><i class="fas fa-clock"></i></span>
                            <span> { &self.dt } </span>
                        </span>
                    </LevelItem>
                </LevelLeft>
                <LevelRight>
                    <LevelItem>
                        <CidExplorer cid={ctx.props().cid} />
                    </LevelItem>
                </LevelRight>
            </Level>
        </Box>
        }
    }

    fn render_comment(&self, ctx: &Context<Self>) -> Html {
        html! {
            <Comment cid={ctx.props().cid} />
        }
    }
}

async fn get_content(ipfs: IpfsService, callback: Callback<(Media, Vec<u8>)>, cid: Cid) {
    let signed_link = match ipfs.dag_get::<&str, SignedLink>(cid, None).await {
        Ok(dag) => dag,
        Err(e) => {
            error!(&format!("{:#?}", e));
            return;
        }
    };

    if !signed_link.verify() {
        error!("Content Verification Failed!");
        return;
    }

    //TODO use hash???
    let pk = signed_link.public_key;

    let media = match ipfs
        .dag_get::<&str, Media>(signed_link.link.link, None)
        .await
    {
        Ok(dag) => dag,
        Err(e) => {
            error!(&format!("{:#?}", e));
            return;
        }
    };

    callback.emit((media, pk));
}
