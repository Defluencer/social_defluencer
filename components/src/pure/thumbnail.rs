#![cfg(target_arch = "wasm32")]

use linked_data::{identity::Identity, media::Media};

use utils::timestamp_to_datetime;

use cid::Cid;

use yew::prelude::*;

use ybc::{
    HeaderSize, ImageSize, Level, LevelItem, LevelLeft, LevelRight, MediaContent, MediaLeft,
    MediaRight, Title,
};

use yew_router::prelude::Link;

use crate::{
    pure::{DagExplorer, IPFSImage},
    Route,
};

#[derive(Properties, PartialEq)]
pub struct ThumbnailProps {
    pub cid: Cid,

    pub media: Media,

    pub identity: Identity,
}

#[function_component(Thumbnail)]
pub fn pure_thumbnail(props: &ThumbnailProps) -> Html {
    let ThumbnailProps {
        cid,
        media,
        identity,
    } = props;
    let cid = *cid;

    let dt = timestamp_to_datetime(media.user_timestamp());

    let content = match media {
        Media::MicroBlog(blog) => {
            let count = words_count::count(&blog.content);

            html! {
            <>
            <Level>
                <LevelLeft>
                    <LevelItem>
                        <span class="icon-text">
                            <span class="icon"><i class="fas fa-user"></i></span>
                            <span> { &identity.display_name } </span>
                        </span>
                    </LevelItem>
                    <LevelItem>
                        <span class="icon-text">
                            <span class="icon"><i class="fa-solid fa-message"></i></span>
                            <span> { format!("{} Characters", count.characters) } </span>
                        </span>
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
            <Link<Route> to={Route::Content{ cid: cid}} >
                {&blog.content}
            </Link<Route>>
            </>
            }
        }
        Media::Blog(article) => {
            html! {
            <>
            <Level>
                <LevelLeft>
                    <LevelItem>
                        <span class="icon-text">
                            <span class="icon"><i class="fas fa-user"></i></span>
                            <span> { &identity.display_name } </span>
                        </span>
                    </LevelItem>
                    <LevelItem>
                        <span class="icon-text">
                            <span class="icon"><i class="fa-solid fa-newspaper"></i></span>
                            <span> { "XXXX Words" } </span>
                        </span>
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
            <Title classes={classes!("has-text-centered")} size={HeaderSize::Is4} >
                {&article.title }
            </Title>
            <Link<Route> to={Route::Content{ cid: cid}} >
                <IPFSImage key={article.image.link.to_string()} cid={article.image.link} size={ImageSize::Is16by9} rounded=false />
            </Link<Route>>
            </>
            }
        }
        Media::Video(video) => {
            let (hour, minute, second) = utils::seconds_to_timecode(video.duration);

            html! {
            <>
            <Level>
                <LevelLeft>
                    <LevelItem>
                        <span class="icon-text">
                            <span class="icon"><i class="fas fa-user"></i></span>
                            <span> { &identity.display_name } </span>
                        </span>
                    </LevelItem>
                    <LevelItem>
                        <span class="icon-text">
                            <span class="icon"><i class="fas fa-video"></i></span>
                            <span> { &format!("{}:{}:{}", hour, minute, second) } </span>
                        </span>
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
            <Title classes={classes!("has-text-centered")} size={HeaderSize::Is4} >
                {&video.title }
            </Title>
            <Link<Route> to={Route::Content{ cid: cid}} >
                <IPFSImage key={video.image.link.to_string()} cid={video.image.link} size={ImageSize::Is16by9} rounded=false />
            </Link<Route>>
            </>
            }
        }
        Media::Comment(comment) => {
            let count = words_count::count(&comment.text);

            html! {
            <>
            <Level>
                <LevelLeft>
                    <LevelItem>
                        <span class="icon-text">
                            <span class="icon"><i class="fas fa-user"></i></span>
                            <span> { &identity.display_name } </span>
                        </span>
                    </LevelItem>
                    <LevelItem>
                        <span class="icon-text">
                            <span class="icon"><i class="fa-solid fa-comment"></i></span>
                            <span> { format!("{} Characters", count.characters) } </span>
                        </span>
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
            <Link<Route> to={Route::Content{ cid: cid}} >
                {&comment.text}
            </Link<Route>>
            </>
            }
        }
    };

    html! {
    <ybc::Media>
        <MediaLeft>
        if let Some(ipld) = identity.avatar {
            <IPFSImage cid={ipld.link} size={ImageSize::Is64x64} rounded=true />
        }
        </MediaLeft>
        <MediaContent>
            { content }
        </MediaContent>
        <MediaRight>
            <DagExplorer key={cid.to_string()} {cid} />
        </MediaRight>
    </ybc::Media>
    }
}