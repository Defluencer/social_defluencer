#![cfg(target_arch = "wasm32")]

use linked_data::{identity::Identity, media::Media};

use utils::timestamp_to_datetime;

use cid::Cid;

use yew::prelude::*;

use ybc::{Block, ImageSize, Level, LevelItem, LevelLeft, MediaContent, MediaLeft, MediaRight};

use yew_router::prelude::Link;

use crate::{
    comment_button::CommentButton, dag_explorer::DagExplorer, md_renderer::Markdown, navbar::Route,
    pure::IPFSImage, share_button::ShareButton, video_player::VideoPlayer,
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

    if let Some(addr) = identity.channel_ipns {
        name = html! {
            <Link<Route> to={Route::Channel{ addr: addr.into()}} >
                {name}
            </Link<Route>>
        };
    }

    let avatar = if let Some(ipld) = identity.avatar {
        html! { <IPFSImage cid={ipld.link} size={ImageSize::Is64x64} rounded=true /> }
    } else {
        html!()
    };

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
                <VideoPlayer {cid}/>
                <Level>
                    <LevelLeft>
                        <LevelItem>
                            {&video.title }
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
            { avatar }
        </MediaLeft>
        <MediaContent>
            <Level>
                <LevelLeft>
                    <LevelItem>
                        { name }
                    </LevelItem>
                    <LevelItem>
                        <span class="icon-text">
                            <span class="icon"><i class="fas fa-clock"></i></span>
                            <span> { dt } </span>
                        </span>
                    </LevelItem>
                </LevelLeft>
            </Level>
            { content }
            { children.clone() }
        </MediaContent>
        <MediaRight>
            <Block>
                <DagExplorer {cid} />
            </Block>
            <Level>
                <LevelItem>
                    <CommentButton {cid} />
                </LevelItem>
                <LevelItem>
                    <ShareButton {cid} />
                </LevelItem>
            </Level>
        </MediaRight>
    </ybc::Media>
        }
}
