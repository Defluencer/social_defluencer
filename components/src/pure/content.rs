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

    pub verified: Option<bool>,

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
        verified,
        children,
    } = props;
    let cid = *cid;

    let is_author = Some(media.identity()) == user_addr;

    let mut name = html! {
        <span class="icon-text">
            <span class="icon"><i class="fas fa-user"></i></span>
            <span><strong>{ &identity.name }</strong></span>
        </span>
    };

    if !is_author {
        if let Some(addr) = identity.ipns_addr {
            name = html! {
                <Link<Route> to={Route::Channel{ addr }} >
                    {name}
                </Link<Route>>
            };
        }
    }

    let content = match media {
        Media::Blog(article) => {
            html! {
                <ybc::Content>
                    <Markdown cid={article.content.link}/>
                </ybc::Content>
            }
        }
        Media::Video(video) => {
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
                        {
                            if let Some(duration) = video.duration {
                                let (hour, minute, second) = utils::seconds_to_timecode(duration);

                                html!{
                                <LevelItem>
                                    <span class="icon-text">
                                        <span class="icon"><i class="fas fa-video"></i></span>
                                        <span><small>{ &format!("{}:{}:{}", hour, minute, second) }</small></span>
                                    </span>
                                </LevelItem>
                                }
                            } else {
                                html!{}
                            }
                        }
                    </LevelLeft>
                </Level>
            </>
            }
        }
        Media::Comment(comment) => html! {<ybc::Content>{&comment.text}</ybc::Content>},
    };

    let dt = timestamp_to_datetime(media.user_timestamp());

    let mut check = html!();

    if verified.is_some() && verified.unwrap() {
        check = html! {
        <LevelItem>
            <span class="icon">
                <i class="fa-solid fa-check"></i>
            </span>
        </LevelItem>
        };
    }

    html! {
    <ybc::Media>
        <MediaLeft>
        if let Some(ipld) = identity.avatar {
            <IPFSImage cid={ipld.link} size={ImageSize::Is64x64} rounded=true />
        }
        </MediaLeft>
        <MediaContent>
            <Block>
                <Level>
                    <LevelLeft>
                        <LevelItem>
                            { name }
                        </LevelItem>
                        {check}
                        <LevelItem>
                            <span class="icon-text">
                                <span class="icon"><i class="fas fa-clock"></i></span>
                                <span><small>{ dt }</small></span>
                            </span>
                        </LevelItem>
                    </LevelLeft>
                    <LevelRight>
                        <LevelItem>
                            <span class="icon-text">
                                <span class="icon"><i class="fa-solid fa-fingerprint"></i></span>
                                <span><small>{cid.to_string()}</small></span>
                            </span>
                        </LevelItem>
                    </LevelRight>
                </Level>
            </Block>
            <Block>
                { content }
            </Block>
            <Block>
                <Level>
                    <LevelLeft>
                        <LevelItem>
                            <CommentButton {cid} identity={identity.clone()} >
                                <Thumbnail key={cid.to_string()} {cid} media={media.clone()} identity={identity.clone()} />
                            </CommentButton>
                        </LevelItem>
                        if !is_author {
                        <LevelItem>
                            <ShareButton {cid} >
                                <Thumbnail key={cid.to_string()} {cid} media={media.clone()} identity={identity.clone()} />
                            </ShareButton>
                        </LevelItem>
                        }
                    </LevelLeft>
                </Level>
            </Block>
            <Block>
                { children.clone() }
            </Block>
        </MediaContent>
        <MediaRight>
            <DagExplorer key={cid.to_string()} {cid} />
        </MediaRight>
    </ybc::Media>
        }
}
