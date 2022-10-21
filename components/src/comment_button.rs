#![cfg(target_arch = "wasm32")]

use cid::Cid;

use defluencer::{
    channel::{local::LocalUpdater, Channel},
    crypto::signers::EthereumSigner,
    user::User,
};

use gloo_console::error;

use ipfs_api::IpfsService;
use linked_data::{
    channel::ChannelMetadata, comments::Comment, identity::Identity, types::IPNSAddress,
};

use utils::{
    commentary::CommentaryContext,
    defluencer::{ChannelContext, UserContext},
    ipfs::IPFSContext,
};

use ybc::{Box, Button, Control, Field, TextArea};

use yew::{platform::spawn_local, prelude::*};

#[derive(Properties, PartialEq)]
pub struct Props {
    /// Signed link to media Cid
    pub cid: Cid,

    pub identity: Identity,

    #[prop_or_default]
    pub children: Children,
}

pub struct CommentButton {
    user_context: Option<UserContext>,

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
        let user_context = match ctx.link().context::<UserContext>(Callback::noop()) {
            Some((context, _)) => Some(context),
            None => None,
        };

        let modal_cb = ctx.link().callback(|_| Msg::Modal);
        let text_cb = ctx.link().callback(Msg::Text);
        let create_cb = ctx.link().callback(|_| Msg::Create);

        Self {
            user_context,

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
        html! {
        <>
        <Button classes={classes!("is-outlined")} onclick={self.modal_cb.clone()} disabled={self.user_context.is_none()} >
            <span class="icon">
                <i class="fa-solid fa-comment"></i>
            </span>
        </Button>
        <div class= { if self.modal { "modal is-active" } else { "modal" } } >
            <div class="modal-background" onclick={self.modal_cb.clone()} ></div>
            <div class="modal-content is-clipped">
                <Box>
                    { ctx.props().children.clone() }
                </Box>
                <Box>
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
                </Box>
            </div>
            <button class="modal-close is-large" aria-label="close" onclick={self.modal_cb.clone()} />
        </div>
        </>
        }
    }
}

impl CommentButton {
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
        let user = match self.user_context.as_ref() {
            Some(cntx) => cntx.user.clone(),
            None => return false,
        };

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

        if let Some((context, _)) = ctx.link().context::<IPFSContext>(Callback::noop()) {
            if let Some(addr) = ctx.props().identity.ipns_addr {
                spawn_local(send_comment(context.client, addr, cid));
            }
        }

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

async fn send_comment(ipfs: IpfsService, addr: IPNSAddress, cid: Cid) {
    let root = match ipfs.name_resolve(addr.into()).await {
        Ok(cid) => cid,
        Err(e) => {
            error!(&format!("{:#?}", e));
            return;
        }
    };

    let meta = match ipfs.dag_get::<&str, ChannelMetadata>(root, None).await {
        Ok(meta) => meta,
        Err(e) => {
            error!(&format!("{:#?}", e));
            return;
        }
    };

    if let Some(topic) = meta.agregation_channel {
        if let Err(e) = ipfs.pubsub_pub(topic, cid.to_bytes()).await {
            error!(&format!("{:#?}", e));
        }
    }
}
