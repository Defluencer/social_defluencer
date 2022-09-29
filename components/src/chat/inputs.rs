#![cfg(target_arch = "wasm32")]

use cid::Cid;

use gloo_console::{error, info};

use linked_data::{
    identity::Identity,
    media::chat::{ChatInfo, ChatMessage, MessageType},
};

use utils::{defluencer::UserContext, ipfs::IPFSContext, web3::Web3Context};

use ybc::{Button, ButtonRouter, Control, Field, Input, TextArea};

use yew::{platform::spawn_local, prelude::*};

use crate::{chat::window::LiveContext, Route};

pub struct ChatInputs {
    name: String,

    message: String,

    signature: Option<Cid>,
}

pub enum Msg {
    Name(String),
    ConfirmName,
    Identity((Cid, Identity)),
    Message(String),
    Enter,
    Signature(Cid),
}

impl Component for ChatInputs {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        #[cfg(debug_assertions)]
        info!("Chat Inputs Created");

        let mut name = String::default();

        if let Some((context, _)) = ctx.link().context::<Web3Context>(Callback::noop()) {
            if let Some(ens_name) = context.name {
                name = ens_name;
            }
        }

        if name.is_empty() {
            if let Some(ipld) = utils::identity::get_current_identity() {
                let (context, _) = ctx
                    .link()
                    .context::<IPFSContext>(Callback::noop())
                    .expect("IPFS Context");

                spawn_local(utils::r#async::get_identity(
                    context.client,
                    ipld.link,
                    ctx.link().callback(Msg::Identity),
                ))
            }
        }

        Self {
            name,
            message: Default::default(),

            signature: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Name(name) => self.on_name_input(name),
            Msg::ConfirmName => self.on_name_confirm(ctx),
            Msg::Identity((_, identity)) => self.on_name_input(identity.display_name),
            Msg::Message(msg) => self.on_chat_input(ctx, msg),
            Msg::Enter => self.send_message(ctx),
            Msg::Signature(cid) => self.on_signature(cid),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if ctx
            .link()
            .context::<UserContext>(Callback::noop())
            .is_none()
        {
            return self.connect_dialog();
        }

        if self.signature.is_none() {
            return self.name_dialog(ctx);
        }

        self.chat_dialog(ctx)
    }
}

impl ChatInputs {
    fn connect_dialog(&self) -> Html {
        html! {
            <Field label={"To chat, please create an identity".to_owned()} >
                <ButtonRouter<Route> route={Route::Settings}>
                    <span class="icon-text">
                        <span class="icon"><i class="fas fa-rss"></i></span>
                        <span> {"Go To Settings"} </span>
                    </span>
                </ButtonRouter<Route>>
            </Field>
        }
    }

    fn name_dialog(&self, ctx: &Context<Self>) -> Html {
        html! {
        <>
        <Field label={"Display Name".to_owned()} >
            <Control>
                <Input name={"chat_name"} value={self.name.clone()} update={ctx.link().callback(Msg::Name)} />
            </Control>
        </Field>
        <Field label={"Confirm your name by signing it".to_owned()} >
            <Control>
                <Button classes={classes!("is-primary")} onclick={ctx.link().callback(|_| Msg::ConfirmName)} >
                    { "Sign" }
                </Button>
            </Control>
        </Field>
        </>
        }
    }

    fn chat_dialog(&self, ctx: &Context<Self>) -> Html {
        html! {
        <>
        <Field>
            <Control>
                <TextArea name={"chat_msg"} value={String::default()} update={ctx.link().callback(Msg::Message)} rows={3} fixed_size=true />
            </Control>
        </Field>
        <Field>
            <Control>
                <Button classes={classes!("is-primary")} onclick={ctx.link().callback(|_| Msg::Enter)} >
                    { "Send" }
                </Button>
            </Control>
        </Field>
        </>
        }
    }

    /// Send chat message via gossipsub.
    fn send_message(&mut self, ctx: &Context<Self>) -> bool {
        let cid = match self.signature {
            Some(cid) => cid,
            None => {
                error!("No Message Signature");
                return false;
            }
        };

        let text = self.message.clone();

        let chat_msg = ChatMessage {
            message: MessageType::Text(text),
            signature: cid.into(),
        };

        let data = match serde_json::to_vec(&chat_msg) {
            Ok(data) => data,
            Err(e) => {
                error!(&format!("{:#?}", e));
                return false;
            }
        };

        #[cfg(debug_assertions)]
        info!("Publish Message");

        let (context, _) = ctx
            .link()
            .context::<IPFSContext>(Callback::noop())
            .expect("IPFS Context");
        let ipfs = context.client;

        let (context, _) = ctx
            .link()
            .context::<LiveContext>(Callback::noop())
            .expect("Live Context");

        let topic = context.settings.chat_topic.unwrap();

        spawn_local({
            async move {
                if let Err(e) = ipfs.pubsub_pub(topic, data).await {
                    error!(&format!("{:#?}", e));
                }
            }
        });

        true
    }

    fn on_chat_input(&mut self, ctx: &Context<Self>, msg: String) -> bool {
        if msg.ends_with('\n') {
            self.message = msg;

            return self.send_message(ctx);
        }

        self.message = msg;

        false
    }

    fn on_name_input(&mut self, name: String) -> bool {
        self.name = name;

        false
    }

    /// Callback when the chat name choice is submited.
    fn on_name_confirm(&mut self, ctx: &Context<Self>) -> bool {
        #[cfg(debug_assertions)]
        info!("Name Submitted");

        let name = self.name.clone();

        let (context, _) = ctx
            .link()
            .context::<IPFSContext>(Callback::noop())
            .expect("IPFS Context");
        let node = context.peer_id;

        let data = ChatInfo { node, name };

        let (context, _) = ctx
            .link()
            .context::<UserContext>(Callback::noop())
            .expect("User Context");
        let user = context.user;

        spawn_local({
            let cb = ctx.link().callback(Msg::Signature);

            async move {
                match user.chat_signature(data).await {
                    Ok(cid) => cb.emit(cid),
                    Err(e) => error!(&format!("{:#?}", e)),
                }
            }
        });

        false
    }

    ///Callback when the chat signature CID is received.
    fn on_signature(&mut self, cid: Cid) -> bool {
        #[cfg(debug_assertions)]
        info!("Message Signature Added");

        self.signature = Some(cid);

        true
    }
}
