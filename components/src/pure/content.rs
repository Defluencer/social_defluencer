#![cfg(target_arch = "wasm32")]

use linked_data::{identity::Identity, media::Media, types::IPLDLink};

use utils::{defluencer::UserContext, timestamp_to_datetime};

use cid::Cid;

use yew::prelude::*;

use ybc::{
    Block, HeaderSize, ImageSize, Level, LevelItem, LevelLeft, LevelRight, MediaContent, MediaLeft,
    MediaRight, Title,
};

use yew_router::prelude::Link;

use crate::{
    comment_button::CommentButton,
    md_renderer::Markdown,
    pure::{DagExplorer, IPFSImage, Thumbnail},
    share_button::ShareButton,
    video_player::VideoPlayer,
    Route,
};

#[derive(Properties, PartialEq)]
pub struct ContentProps {
    pub cid: Cid,

    pub media: Media,

    pub identity: Identity,

    #[prop_or_default]
    pub children: Children,
}

#[function_component(Content)]
pub fn pure_content(props: &ContentProps) -> Html {
    let user_addr: Option<IPLDLink> =
        use_context::<UserContext>().map(|context| context.user.get_identity().into());

    let ContentProps {
        cid,
        media,
        identity,
        children,
    } = props;
    let cid = *cid;

    let mut name = html! {
        <span class="icon-text">
            <span class="icon"><i class="fas fa-user"></i></span>
            <span> { &identity.display_name } </span>
        </span>
    };

    if Some(media.identity()) != user_addr {
        if let Some(addr) = identity.channel_ipns {
            name = html! {
                <Link<Route> to={Route::Channel{ addr: addr.into()}} >
                    {name}
                </Link<Route>>
            };
        }
    }

    let content = match media {
        Media::MicroBlog(blog) => html! {<ybc::Content>{&blog.content}</ybc::Content>},
        Media::Blog(article) => {
            html! {
                <ybc::Content>
                    <Markdown cid={article.content.link}/>
                </ybc::Content>
            }
        }
        Media::Video(video) => {
            let (hour, minute, second) = utils::seconds_to_timecode(video.duration);

            html! {
            <>
                <Block>
                    <VideoPlayer {cid}/>
                </Block>
                <Level>
                    <LevelLeft>
                        <LevelItem>
                            <Title classes={classes!("has-text-centered")} size={HeaderSize::Is4} >
                                {&video.title }
                            </Title>
                        </LevelItem>
                        <LevelItem>
                            <span class="icon-text">
                                <span class="icon"><i class="fas fa-video"></i></span>
                                <span> { &format!("{}:{}:{}", hour, minute, second) } </span>
                            </span>
                        </LevelItem>
                    </LevelLeft>
                </Level>
            </>
            }
        }
        Media::Comment(comment) => html! {<ybc::Content>{&comment.text}</ybc::Content>},
    };

    let dt = timestamp_to_datetime(media.user_timestamp());

    html! {
    <ybc::Media>
        <MediaLeft>
        if let Some(ipld) = identity.avatar {
            <IPFSImage cid={ipld.link} size={ImageSize::Is64x64} rounded=true />
        }
        </MediaLeft>
        <MediaContent>
            <Level>
                <LevelLeft>
                    <LevelItem>
                        { name }
                    </LevelItem>
                </LevelLeft>
                <LevelRight>
                    <LevelItem>
                        <span class="icon-text">
                            <span class="icon"><i class="fas fa-clock"></i></span>
                            <span> { dt } </span>
                        </span>
                    </LevelItem>
                </LevelRight>
            </Level>
            { content }
            <Level>
                <LevelLeft>
                    <LevelItem>
                        <CommentButton {cid} >
                            <Thumbnail key={cid.to_string()} {cid} media={media.clone()} identity={identity.clone()} />
                        </CommentButton>
                    </LevelItem>
                    if Some(media.identity()) != user_addr {
                        <LevelItem>
                            <ShareButton {cid} >
                                <Thumbnail key={cid.to_string()} {cid} media={media.clone()} identity={identity.clone()} />
                            </ShareButton>
                        </LevelItem>
                    }
                </LevelLeft>
            </Level>
            { children.clone() }
        </MediaContent>
        <MediaRight>
            <DagExplorer key={cid.to_string()} {cid} />
        </MediaRight>
    </ybc::Media>
        }
}