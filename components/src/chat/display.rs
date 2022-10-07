#![cfg(target_arch = "wasm32")]

use std::collections::{HashMap, VecDeque};

use cid::Cid;

use defluencer::crypto::signed_link::SignedLink;

use futures_util::{
    stream::{AbortHandle, Abortable},
    StreamExt,
};

use gloo_console::error;

use ipfs_api::IpfsService;

use linked_data::{
    media::chat::{ChatInfo, ChatMessage, MessageType},
    types::PeerId,
};

use utils::ipfs::IPFSContext;

use yew::{platform::spawn_local, prelude::*};

use super::window::LiveContext;

const MAX_MSG_NUM: usize = 50;

pub struct ChatDisplay {
    handle: AbortHandle,

    verified_sigs: HashMap<Cid, ChatInfo>,

    pending_verifs: HashMap<Cid, String>,

    messages: VecDeque<Html>,
}

pub enum Msg {
    PubSub((Cid, Vec<u8>)),
    Verification((Cid, ChatInfo)),
}

impl Component for ChatDisplay {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (live_context, _) = ctx
            .link()
            .context::<LiveContext>(Callback::noop())
            .expect("Live Context");

        let topic = live_context.settings.chat_topic.unwrap();

        let (ipfs_context, _) = ctx
            .link()
            .context::<IPFSContext>(Callback::noop())
            .expect("IPFS Context");

        let (handle, regis) = AbortHandle::new_pair();

        spawn_local({
            let ipfs = ipfs_context.client.clone();
            let cb = ctx.link().callback(Msg::PubSub).clone();
            let topic = topic.clone();

            async move {
                let stream = ipfs.pubsub_sub(topic.into_bytes());

                let mut stream = Abortable::new(stream, regis).boxed_local();

                while let Some(result) = stream.next().await {
                    match result {
                        Ok(msg) => cb.emit((msg.from, msg.data)),
                        Err(e) => error!(&format!("{:#?}", e)),
                    }
                }
            }
        });

        Self {
            handle,
            verified_sigs: Default::default(),
            pending_verifs: Default::default(),
            messages: VecDeque::with_capacity(MAX_MSG_NUM),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::PubSub((from, data)) => self.on_message(ctx, from, data),
            Msg::Verification((cid, info)) => self.on_verif(cid, info),
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let messages = self
            .messages
            .iter()
            .map(|item| item.clone())
            .collect::<Html>();

        html! {
        <div id="chat_display" class="box" style="overflow-y: scroll;height: 60vh;scroll-behavior: smooth;" >
            { messages }
        </div>
        }
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        self.handle.abort();
    }
}

impl ChatDisplay {
    /// Callback when a message is received
    fn on_message(&mut self, ctx: &Context<Self>, from: Cid, data: Vec<u8>) -> bool {
        let peer_id = match PeerId::try_from(from) {
            Ok(peer) => peer,
            Err(e) => {
                error!(&format!("{:#?}", e));
                return false;
            }
        };

        let msg = match serde_json::from_slice::<ChatMessage>(&data) {
            Ok(msg) => msg,
            Err(e) => {
                error!(&format!("{:#?}", e));
                return false;
            }
        };

        let ChatMessage { message, signature } = msg;

        let text = match message {
            MessageType::Text(text) => text,
            _ => return false, //TODO process moderation messages
        };

        if let Some(info) = self.verified_sigs.get(&signature.link) {
            if info.node == peer_id {
                let msg = message_html(&info.name, &text);

                self.messages.push_back(msg);

                if self.messages.len() > MAX_MSG_NUM {
                    self.messages.pop_front();
                }

                return true;
            }
        }

        self.pending_verifs.insert(signature.link, text);

        let (context, _) = ctx
            .link()
            .context::<IPFSContext>(Callback::noop())
            .expect("IPFS Context");

        spawn_local(user_verification(
            context.client.clone(),
            peer_id,
            signature.link,
            ctx.link().callback(Msg::Verification),
        ));

        false
    }

    /// Callback when user verification is complete
    fn on_verif(&mut self, sig: Cid, info: ChatInfo) -> bool {
        if let Some(text) = self.pending_verifs.remove(&sig) {
            let msg = message_html(&info.name, &text);

            self.messages.push_back(msg);

            if self.messages.len() > MAX_MSG_NUM {
                self.messages.pop_front();
            }
        }

        self.verified_sigs.insert(sig, info);

        false
    }
}

fn message_html(name: &str, text: &str) -> Html {
    html! {
        <article class="message is-small" style="overflow-wrap: break-word" >
            <ybc::MessageHeader>
                //<ybc::Image size={ybc::ImageSize::IsSquare} >
                    //<img src={props.img_url.clone()} height="32" width="32" />
                //</ybc::Image>
                <h3>{ name }</h3>
            </ybc::MessageHeader>
            <ybc::MessageBody>
                { text }
            </ybc::MessageBody>
        </article>
    }
}

async fn user_verification(
    ipfs: IpfsService,
    peer: PeerId,
    sig: Cid,
    callback: Callback<(Cid, ChatInfo)>,
) {
    //TODO once Ledger app is built switch to DAG-JOSE
    /* let jws: JsonWebSignature = match ipfs.dag_get::<&str, RawJWS>(msg.signature.link, None).await {
        Ok(dag) => match dag.try_into() {
            Ok(dag) => dag,
            Err(e) => {
                error!(&format!("{:#?}", e));
                return;
            }
        },
        Err(e) => {
            error!(&format!("{:#?}", e));
            return;
        }
    };

    if let Err(e) = jws.verify() {
        error!(&format!("{:#?}", e));
        return;
    } */

    let signed_link = match ipfs.dag_get::<&str, SignedLink>(sig, None).await {
        Ok(dag) => dag,
        Err(e) => {
            error!(&format!("{:#?}", e));
            return;
        }
    };

    if !signed_link.verify() {
        error!("Cannot Verify Signature");
        return;
    }

    let chat_info = match ipfs
        .dag_get::<&str, ChatInfo>(signed_link.link.link, None)
        .await
    {
        Ok(dag) => dag,
        Err(e) => {
            error!(&format!("{:#?}", e));
            return;
        }
    };

    if chat_info.node != peer {
        error!("Cannot Verify Chat Sender");
        return;
    }

    callback.emit((sig, chat_info));
}
