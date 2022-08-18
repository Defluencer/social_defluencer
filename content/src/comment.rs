#![cfg(target_arch = "wasm32")]

use components::cid_explorer::CidExplorer;

use utils::timestamp_to_datetime;

use cid::Cid;

use utils::ipfs::IPFSContext;

use yew::{platform::spawn_local, prelude::*};

use gloo_console::error;

use ybc::{Block, Box, Content, Media, MediaContent, MediaLeft, MediaRight};

use components::identification::Identification;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub cid: Cid,
}

pub struct Comment {
    dt: String,
    comment: Option<linked_data::comments::Comment>,
}

pub enum Msg {
    Comment(linked_data::comments::Comment),
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
            let cb = ctx.link().callback(Msg::Comment);
            let ipfs = context.client.clone();
            let cid = ctx.props().cid;

            async move {
                match ipfs
                    .dag_get::<String, linked_data::comments::Comment>(cid, None)
                    .await
                {
                    Ok(comment) => cb.emit(comment),
                    Err(e) => error!(&format!("{:#?}", e)),
                }
            }
        });

        Self {
            dt: String::new(),
            comment: None,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Comment(comment) => {
                self.dt = timestamp_to_datetime(comment.user_timestamp);
                self.comment = Some(comment);

                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        match &self.comment {
            Some(comment) => {
                html! {
                <Box>
                    <Media>
                        <MediaLeft>
                            <Identification cid={comment.identity.link} />
                            <Block>
                                <span class="icon-text">
                                    <span class="icon"><i class="fas fa-clock"></i></span>
                                    <span> { &self.dt } </span>
                                </span>
                            </Block>
                        </MediaLeft>
                        <MediaContent>
                            <Content classes={classes!("has-text-centered")} >
                                { &comment.text }
                            </Content>
                        </MediaContent>
                        <MediaRight>
                            <CidExplorer cid={comment.origin} />
                        </MediaRight>
                    </Media>
                </Box>
                }
            }
            None => html! {
                <span class="bulma-loader-mixin"></span>
            },
        }
    }
}
