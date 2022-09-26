#![cfg(target_arch = "wasm32")]

use cid::Cid;

use crate::{identification::Identification, thumbnail::Thumbnail};

use defluencer::{
    channel::{local::LocalUpdater, Channel},
    crypto::signers::EthereumSigner,
    user::User,
};

use gloo_console::error;

use linked_data::comments::Comment;

use utils::{
    commentary::CommentaryContext,
    defluencer::{ChannelContext, UserContext},
};

use ybc::{Box, Button, Control, Field, Media, MediaContent, MediaLeft, TextArea};

use yew::{platform::spawn_local, prelude::*};

#[derive(Properties, PartialEq)]
pub struct Props {
    /// Signed link to media Cid
    pub cid: Cid,
}

pub struct CommentButton {
    identity_cid: Option<Cid>,

    text: String,
    text_cb: Callback<String>,

    create_cb: Callback<MouseEvent>,

    modal_cb: Callback<MouseEvent>,
    modal: bool,
    loading: bool,
}

pub enum Msg {
    Modal,
    Text(String),
    Create,
    Done(Cid),
}

impl Component for CommentButton {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let identity_cid = match ctx.link().context::<UserContext>(Callback::noop()) {
            Some((context, _)) => Some(context.user.get_identity()),
            None => None,
        };

        let modal_cb = ctx.link().callback(|_| Msg::Modal);
        let text_cb = ctx.link().callback(Msg::Text);
        let create_cb = ctx.link().callback(|_| Msg::Create);

        Self {
            identity_cid,

            text: String::default(),
            text_cb,

            create_cb,

            modal_cb,
            modal: false,
            loading: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Modal => self.on_click(),
            Msg::Text(text) => self.on_text(text),
            Msg::Create => self.on_create(ctx),
            Msg::Done(cid) => self.on_done(cid, ctx),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if self.identity_cid.is_none() {
            return html! {
            <Button disabled={true} >
                <span class="icon">
                    <i class="fa-solid fa-comment"></i>
                </span>
            </Button>
            };
        }

        let cid = self.identity_cid.unwrap();

        html! {
        <>
            <Button classes={classes!("is-outlined")} onclick={self.modal_cb.clone()} >
                <span class="icon">
                    <i class="fa-solid fa-comment"></i>
                </span>
            </Button>
            { self.render_modal(cid, ctx) }
        </>
        }
    }
}

impl CommentButton {
    fn render_modal(&self, cid: Cid, ctx: &Context<Self>) -> Html {
        html! {
        <div class= { if self.modal { "modal is-active" } else { "modal" } } >
            <div class="modal-background" onclick={self.modal_cb.clone()} ></div>
            <div class="modal-content">
                <Box>
                    <Thumbnail cid={ctx.props().cid} />
                </Box>
                <Box>
                    <Media>
                        <MediaLeft>
                            <Identification {cid} />
                        </MediaLeft>
                        <MediaContent>
                            <Field>
                                <Control>
                                    <TextArea name="text" value="" update={self.text_cb.clone()} placeholder={"Add a comment..."} rows={4} fixed_size={true} />
                                </Control>
                            </Field>
                            <Field>
                                <Control>
                                    <Button onclick={self.create_cb.clone()} loading={self.loading} >
                                        { "Post" }
                                    </Button>
                                </Control>
                            </Field>
                        </MediaContent>
                    </Media>
                </Box>
            </div>
            <button class="modal-close is-large" aria-label="close" onclick={self.modal_cb.clone()} />
        </div>
        }
    }

    fn on_click(&mut self) -> bool {
        self.modal = !self.modal;

        true
    }

    fn on_text(&mut self, text: String) -> bool {
        if self.text == text {
            return false;
        }

        self.text = text;

        false
    }

    fn on_create(&mut self, ctx: &Context<Self>) -> bool {
        let (context, _) = ctx
            .link()
            .context::<UserContext>(Callback::noop())
            .expect("User Context");

        let user = context.user;

        let origin = ctx.props().cid;
        let text = self.text.clone();

        let parent_cb = ctx
            .link()
            .context::<CommentaryContext>(Callback::noop())
            .map(|(context, _)| context.callback);

        let done_cb = ctx.link().callback(Msg::Done);

        spawn_local(publish_comment(user, origin, text, parent_cb, done_cb));

        self.loading = true;

        true
    }

    fn on_done(&mut self, cid: Cid, ctx: &Context<Self>) -> bool {
        self.loading = false;
        self.modal = false;

        if let Some((context, _)) = ctx.link().context::<ChannelContext>(Callback::noop()) {
            spawn_local(add_to_channel(context.channel, cid));
        }

        //TODO send comment cid to original channel for aggregation

        true
    }
}

async fn publish_comment(
    user: User<EthereumSigner>,
    origin: Cid,
    text: String,
    parent_cb: Option<Callback<(Cid, Comment)>>,
    done_cb: Callback<Cid>,
) {
    match user.create_comment(origin, text, false).await {
        Ok((cid, comment)) => {
            if let Some(callback) = parent_cb {
                callback.emit((cid, comment));
            }

            done_cb.emit(cid);
        }
        Err(e) => error!(&format!("{:#?}", e)),
    }
}

async fn add_to_channel(channel: Channel<LocalUpdater>, cid: Cid) {
    if let Err(e) = channel.add_comment(cid).await {
        error!(&format!("{:#?}", e))
    }
}
