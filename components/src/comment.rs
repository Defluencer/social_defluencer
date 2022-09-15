#![cfg(target_arch = "wasm32")]

use linked_data::identity::Identity;

use utils::timestamp_to_datetime;

use cid::Cid;

use utils::ipfs::IPFSContext;

use yew::{platform::spawn_local, prelude::*};

use gloo_console::error;

use ybc::{
    Block, Content, ImageSize, Level, LevelItem, LevelLeft, Media, MediaContent, MediaLeft,
    MediaRight,
};

use crate::{
    comment_button::CommentButton, dag_explorer::DagExplorer, image::Image, searching::Searching,
    share_button::ShareButton,
};

#[derive(Properties, PartialEq)]
pub struct Props {
    /// Signed link to media Cid
    pub cid: Cid,
}

pub struct Comment {
    dt: String,
    comment: Option<linked_data::comments::Comment>,

    identity: Option<Identity>,
}

pub enum Msg {
    Comment(linked_data::comments::Comment),
    Identity(Identity),
}

impl Component for Comment {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let (context, _) = ctx
            .link()
            .context::<IPFSContext>(Callback::noop())
            .expect("IPFS Context");

        spawn_local({
            let comment_cb = ctx.link().callback(Msg::Comment);
            let identity_cb = ctx.link().callback(Msg::Identity);
            let ipfs = context.client.clone();
            let cid = ctx.props().cid;

            async move {
                let comment = match ipfs
                    .dag_get::<&str, linked_data::comments::Comment>(cid, Some("/link"))
                    .await
                {
                    Ok(comment) => comment,
                    Err(e) => {
                        error!(&format!("{:#?}", e));
                        return;
                    }
                };

                let cid = comment.identity.link;
                comment_cb.emit(comment);

                match ipfs.dag_get::<&str, Identity>(cid, None).await {
                    Ok(id) => identity_cb.emit(id),
                    Err(e) => error!(&format!("{:#?}", e)),
                }
            }
        });

        Self {
            dt: String::new(),
            comment: None,
            identity: None,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Comment(comment) => {
                self.dt = timestamp_to_datetime(comment.user_timestamp);
                self.comment = Some(comment);

                true
            }
            Msg::Identity(id) => {
                self.identity = Some(id);

                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if self.comment.is_none() || self.identity.is_none() {
            return html! {
                <Searching />
            };
        }

        let comment = self.comment.as_ref().unwrap();
        let identity = self.identity.as_ref().unwrap();

        html! {
        <Media>
            <MediaLeft>
            {
                if let Some(ipld) = identity.avatar {
                    html! {
                    <ybc::Image size={ImageSize::Is64x64} >
                        <Image key={ipld.link.to_string()} cid={ipld.link} />
                    </ybc::Image>
                    }
                } else {
                    html!()
                }
            }
            </MediaLeft>
            <MediaContent>
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
                                <span class="icon"><i class="fas fa-clock"></i></span>
                                <span> { &self.dt } </span>
                            </span>
                        </LevelItem>
                    </LevelLeft>
                </Level>
                <Content>
                    { &comment.text }
                </Content>
            </MediaContent>
            <MediaRight>
                <Block>
                    <DagExplorer cid={ctx.props().cid} />
                </Block>
                <Level>
                    <LevelItem>
                        <CommentButton cid={ctx.props().cid} />
                    </LevelItem>
                    <LevelItem>
                        <ShareButton cid={ctx.props().cid} />
                    </LevelItem>
                </Level>
            </MediaRight>
        </Media>
        }
    }
}
