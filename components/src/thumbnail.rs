#![cfg(target_arch = "wasm32")]

use cid::Cid;

use gloo_console::error;

use linked_data::{
    identity::Identity,
    media::{
        blog::{FullPost, MicroPost},
        video::Video,
        Media,
    },
};

use utils::ipfs::IPFSContext;

use ybc::{ImageSize, Level, LevelItem, LevelLeft, MediaContent, MediaLeft, Title};

use yew::{platform::spawn_local, prelude::*};

use yew_router::prelude::Link;

use crate::{navbar::Route, pure::IPFSImage};

#[derive(Properties, PartialEq)]
pub struct Props {
    /// Signed link to Media Content Cid
    pub cid: Cid,
}

pub struct Thumbnail {
    media: Option<Media>,
    identity: Option<Identity>,
}

pub enum Msg {
    Metadata(Media),
    Identity(Identity),
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
            let media_cb = ctx.link().callback(Msg::Metadata);
            let id_cb = ctx.link().callback(Msg::Identity);
            let ipfs = context.client.clone();
            let cid = ctx.props().cid;

            async move {
                let (media_res, id_res) = futures_util::join!(
                    ipfs.dag_get::<&str, Media>(cid, Some("/link")),
                    ipfs.dag_get::<&str, Identity>(cid, Some("/link/identity"))
                );

                match media_res {
                    Ok(dag) => media_cb.emit(dag),
                    Err(e) => error!(&format!("{:#?}", e)),
                }

                match id_res {
                    Ok(dag) => id_cb.emit(dag),
                    Err(e) => error!(&format!("{:#?}", e)),
                }
            }
        });

        Self {
            media: None,
            identity: None,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Metadata(media) => {
                self.media = Some(media);
            }
            Msg::Identity(identity) => {
                self.identity = Some(identity);
            }
        }

        self.media.is_some() && self.identity.is_some()
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if self.media.is_none() || self.identity.is_none() {
            return html! {};
        }

        let media = self.media.as_ref().unwrap();
        let identity = self.identity.as_ref().unwrap();

        match media {
            Media::Video(metadata) => self.render_video(ctx, metadata, identity),
            Media::Blog(metadata) => self.render_blog(ctx, metadata, identity),
            Media::MicroBlog(metadata) => self.render_microblog(ctx, metadata, identity),
            Media::Comment(metadata) => self.render_comment(ctx, metadata, identity),
        }
    }
}
impl Thumbnail {
    fn render_video(&self, ctx: &Context<Self>, metadata: &Video, identity: &Identity) -> Html {
        html! {
        <ybc::Media>
            <MediaLeft>
            {
                if let Some(ipld) = identity.avatar {
                    html! { <IPFSImage key={ipld.link.to_string()} cid={ipld.link} size={ImageSize::Is64x64} rounded=true /> }
                } else {
                    html!()
                }
            }
            </MediaLeft>
            <MediaContent classes={classes!("has-text-centered")} >
                <Link<Route> to={Route::Content{ cid: ctx.props().cid}} >
                    <Title classes={classes!("is-6")} >
                        { &metadata.title }
                    </Title>
                    <IPFSImage key={metadata.image.link.to_string()} cid={metadata.image.link} size={ImageSize::Is16by9} rounded=false />
                </Link<Route>>
            </MediaContent>
        </ybc::Media>
        }
    }

    fn render_blog(&self, ctx: &Context<Self>, metadata: &FullPost, identity: &Identity) -> Html {
        html! {
        <ybc::Media>
            <MediaLeft>
            {
                if let Some(ipld) = identity.avatar {
                    html! { <IPFSImage key={ipld.link.to_string()} cid={ipld.link} size={ImageSize::Is64x64} rounded=true /> }
                } else {
                    html!()
                }
            }
            </MediaLeft>
            <MediaContent>
                <Level>
                    <LevelLeft>
                        <LevelItem>
                        {
                            if let Some(addr) = identity.channel_ipns {
                                html! {
                                    <Link<Route> to={Route::Channel{ addr: addr.into()}} >
                                        { &identity.display_name }
                                    </Link<Route>>
                                }
                            } else {
                               html! { { &identity.display_name } }
                            }
                        }
                        </LevelItem>
                    </LevelLeft>
                </Level>
                <Link<Route> to={Route::Content{ cid: ctx.props().cid}} >
                    { &metadata.title }
                    <IPFSImage key={metadata.image.link.to_string()} cid={metadata.image.link} size={ImageSize::Is16by9} rounded=false />
                </Link<Route>>
            </MediaContent>
        </ybc::Media>
        }
    }

    fn render_microblog(
        &self,
        ctx: &Context<Self>,
        metadata: &MicroPost,
        identity: &Identity,
    ) -> Html {
        html! {
        <ybc::Media>
            <MediaLeft>
            {
                if let Some(ipld) = identity.avatar {
                    html! { <IPFSImage key={ipld.link.to_string()} cid={ipld.link} size={ImageSize::Is64x64} rounded=true /> }
                } else {
                    html!()
                }
            }
            </MediaLeft>
            <MediaContent>
                <Level>
                    <LevelLeft>
                        <LevelItem>
                        {
                            if let Some(addr) = identity.channel_ipns {
                                html! {
                                    <Link<Route> to={Route::Channel{ addr: addr.into()}} >
                                        { &identity.display_name }
                                    </Link<Route>>
                                }
                            } else {
                               html! { { &identity.display_name } }
                            }
                        }
                        </LevelItem>
                    </LevelLeft>
                </Level>
                <Link<Route> to={Route::Content{ cid: ctx.props().cid}} >
                    { &metadata.content }
                </Link<Route>>
            </MediaContent>
        </ybc::Media>
        }
    }

    fn render_comment(
        &self,
        ctx: &Context<Self>,
        metadata: &linked_data::comments::Comment,
        identity: &Identity,
    ) -> Html {
        html! {
        <ybc::Media>
            <MediaLeft>
            {
                if let Some(ipld) = identity.avatar {
                    html! { <IPFSImage key={ipld.link.to_string()} cid={ipld.link} size={ImageSize::Is64x64} rounded=true /> }
                } else {
                    html!()
                }
            }
            </MediaLeft>
            <MediaContent>
                <Level>
                    <LevelLeft>
                        <LevelItem>
                        {
                            if let Some(addr) = identity.channel_ipns {
                                html! {
                                    <Link<Route> to={Route::Channel{ addr: addr.into()}} >
                                        { &identity.display_name }
                                    </Link<Route>>
                                }
                            } else {
                               html! { { &identity.display_name } }
                            }
                        }
                        </LevelItem>
                    </LevelLeft>
                </Level>
                <Link<Route> to={Route::Content{ cid: ctx.props().cid}} >
                    { &metadata.text }
                </Link<Route>>
            </MediaContent>
        </ybc::Media>
        }
    }
}
