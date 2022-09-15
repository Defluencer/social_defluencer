#![cfg(target_arch = "wasm32")]

use defluencer::crypto::signed_link::SignedLink;

use ipfs_api::IpfsService;

use utils::{ipfs::IPFSContext, timestamp_to_datetime};

use linked_data::media::{
    blog::{FullPost, MicroPost},
    video::Video,
    Media,
};

use ybc::{Box, Container, ImageSize, Level, LevelItem, LevelLeft, LevelRight, Section, Title};

use yew::{platform::spawn_local, prelude::*};

use cid::Cid;

use gloo_console::{error, info};

use components::{
    comment::Comment, comment_button::CommentButton, dag_explorer::DagExplorer,
    identification::Identification, image::Image, searching::Searching, share_button::ShareButton,
    video_player::VideoPlayer,
};

use crate::md_renderer::Markdown;

#[derive(Properties, PartialEq)]
pub struct Props {
    /// Signed link to media Cid
    pub cid: Cid,
}

pub struct Content {
    media: Option<Media>,
    dt: String,
    addr: String,
}

pub enum Msg {
    Media((Media, String)),
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
            addr: String::new(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        #[cfg(debug_assertions)]
        info!("Content Update");

        match msg {
            Msg::Media((media, addr)) => {
                self.dt = timestamp_to_datetime(media.user_timestamp());
                self.media = Some(media);
                self.addr = addr;

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
                            <Searching />
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
            <Level>
                <LevelLeft>
                    <LevelItem>
                        <Identification key={blog.identity.link.to_string()} cid={blog.identity.link} addr={self.addr.clone()} />
                    </LevelItem>
                    <LevelItem>
                        <span class="icon-text">
                            <span class="icon"><i class="fas fa-clock"></i></span>
                            <span> { &self.dt } </span>
                        </span>
                    </LevelItem>
                </LevelLeft>
                <LevelRight>
                    <small>{ ctx.props().cid.to_string() }</small>
                </LevelRight>
            </Level>
            <ybc::Content>
                { &blog.content }
            </ybc::Content>
            <Level>
                <LevelLeft>
                    <LevelItem>
                        <CommentButton cid={ctx.props().cid} />
                    </LevelItem>
                    <LevelItem>
                        <ShareButton cid={ctx.props().cid} />
                    </LevelItem>
                </LevelLeft>
                <LevelRight>
                    <DagExplorer cid={ctx.props().cid} />
                </LevelRight>
            </Level>
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
                        <Identification cid={article.identity.link} addr={self.addr.clone()} />
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
                        <DagExplorer cid={ctx.props().cid} />
                    </LevelItem>
                </LevelRight>
            </Level>
            <ybc::Content>
                <Markdown cid={article.content.link} />
            </ybc::Content>
            <Level>
                <LevelLeft>
                    <LevelItem>
                        <CommentButton cid={ctx.props().cid} />
                    </LevelItem>
                    <LevelItem>
                        <ShareButton cid={ctx.props().cid} />
                    </LevelItem>
                </LevelLeft>
            </Level>
        </Box>
        }
    }

    fn render_video(&self, ctx: &Context<Self>, video: &Video) -> Html {
        html! {
        <Box>
            <Title>
                { &video.title }
            </Title>
            <Level>
                <LevelLeft>
                    <LevelItem>
                        <Identification cid={video.identity.link} addr={self.addr.clone()} />
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
                        <DagExplorer cid={ctx.props().cid} />
                    </LevelItem>
                </LevelRight>
            </Level>
            <VideoPlayer cid={ctx.props().cid} />
            <Level>
                <LevelLeft>
                    <LevelItem>
                        <CommentButton cid={ctx.props().cid} />
                    </LevelItem>
                    <LevelItem>
                        <ShareButton cid={ctx.props().cid} />
                    </LevelItem>
                </LevelLeft>
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

async fn get_content(ipfs: IpfsService, callback: Callback<(Media, String)>, cid: Cid) {
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

    let addr = signed_link.get_address();

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

    callback.emit((media, addr));
}
