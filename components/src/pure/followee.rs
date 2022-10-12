#![cfg(target_arch = "wasm32")]

use linked_data::identity::Identity;

use cid::Cid;

use yew::prelude::*;

use ybc::{Block, Content, ImageSize, Level, LevelItem, LevelLeft, MediaContent, MediaLeft};

use yew_router::prelude::Link;

use crate::{pure::IPFSImage, Route};

#[derive(Properties, PartialEq)]
pub struct FolloweeProps {
    pub cid: Cid,

    pub identity: Identity,
}

#[function_component(Followee)]
pub fn pure_followee(props: &FolloweeProps) -> Html {
    let FolloweeProps { cid: _, identity } = props;

    let addr = identity.ipns_addr.expect("Followee IPNS Address");

    html! {
    <Block>
        <ybc::Media>
            <MediaLeft>
                if let Some(avatar) = identity.avatar {
                    <IPFSImage cid={avatar.link} size={ImageSize::Is64x64} rounded=true />
                }
            </MediaLeft>
            <MediaContent>
                <Level>
                <LevelLeft>
                    <LevelItem>
                        <Link<Route> to={Route::Channel{ addr }} >
                            <span class="icon-text">
                                <span class="icon"><i class="fas fa-user"></i></span>
                                <span><strong>{ &identity.name }</strong></span>
                            </span>
                        </Link<Route>>
                    </LevelItem>
                </LevelLeft>
                </Level>
            if let Some(bio) = &identity.bio {
                <Content>{ bio }</Content>
            }
            </MediaContent>
        </ybc::Media>
    </Block>
    }
}
