#![cfg(target_arch = "wasm32")]

use std::collections::VecDeque;

use components::{
    identification::Identification, navbar::NavigationBar, searching::Searching,
    thumbnail::Thumbnail,
};

use defluencer::Defluencer;

use futures_util::{
    stream::{AbortHandle, AbortRegistration},
    StreamExt,
};

use gloo_console::error;

use ipfs_api::IpfsService;

use linked_data::{
    channel::ChannelMetadata,
    types::{IPLDLink, IPNSAddress},
};

use utils::ipfs::IPFSContext;

use ybc::{Container, Section};

use yew::{platform::spawn_local, prelude::*};

use cid::Cid;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub cid: Cid,
}

//TODO If your channel, add buttons to post & remove stuff

//TODO If live, display video

/// social.defluencer.eth/#/channel/<IPNS_HERE>
pub struct ChannelPage {
    handle: AbortHandle,

    metadata: Option<ChannelMetadata>,

    content: VecDeque<Cid>,
}

pub enum Msg {
    Update(ChannelMetadata),
    Content(Cid),
}

impl Component for ChannelPage {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let (context, _) = ctx
            .link()
            .context::<IPFSContext>(Callback::noop())
            .expect("IPFS Context");

        let (handle, regis) = AbortHandle::new_pair();

        spawn_local(channel_subscribe(
            context.client.clone(),
            ctx.link().callback(Msg::Update),
            ctx.props().cid.into(),
            regis,
        ));

        Self {
            handle,
            metadata: None,

            content: Default::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Update(metadata) => self.on_channel_update(ctx, metadata),
            Msg::Content(cid) => self.on_channel_content(ctx, cid),
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        match &self.metadata {
            Some(meta) => self.render_channel(meta),
            None => self.render_no_channel(),
        }
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        self.handle.abort();
    }
}

impl ChannelPage {
    fn on_channel_update(&mut self, ctx: &Context<Self>, metadata: ChannelMetadata) -> bool {
        if let Some(index) = metadata.content_index {
            self.content.clear();

            let (context, _) = ctx
                .link()
                .context::<IPFSContext>(Callback::noop())
                .expect("IPFS Context");

            spawn_local(stream_content(
                context.client.clone(),
                ctx.link().callback(Msg::Content),
                index,
            ));
        }

        self.metadata = Some(metadata);

        false
    }

    fn on_channel_content(&mut self, _ctx: &Context<ChannelPage>, cid: Cid) -> bool {
        self.content.push_back(cid);

        if self.content.len() > 50 {
            self.content.pop_front();
        }

        true
    }

    fn render_channel(&self, meta: &ChannelMetadata) -> Html {
        html! {
        <>
            <NavigationBar />
            <Section>
                <Container>
                    { "Channel" }
                    <Identification cid={meta.identity.link} />
                    { "Content" }
                    {
                        self.content.iter().map(|&cid| {
                            html! {
                                <Thumbnail {cid} />
                            }
                        }).collect::<Html>()
                    }
                </Container>
            </Section>
        </>
        }
    }

    fn render_no_channel(&self) -> Html {
        html! {
            <Searching />
        }
    }
}

async fn channel_subscribe(
    ipfs: IpfsService,
    callback: Callback<ChannelMetadata>,
    addr: IPNSAddress,
    regis: AbortRegistration,
) {
    let defluencer = Defluencer::new(ipfs.clone());

    let mut stream = defluencer
        .subscribe_channel_updates(addr, regis)
        .boxed_local();

    while let Some(result) = stream.next().await {
        let cid = match result {
            Ok(cid) => cid,
            Err(e) => {
                error!(&format!("{:#?}", e));
                return;
            }
        };

        match ipfs.dag_get::<&str, ChannelMetadata>(cid, None).await {
            Ok(dag) => callback.emit(dag),
            Err(e) => {
                error!(&format!("{:#?}", e));
                return;
            }
        }
    }
}

async fn stream_content(ipfs: IpfsService, callback: Callback<Cid>, index: IPLDLink) {
    let defluencer = Defluencer::new(ipfs.clone());

    let mut stream = defluencer
        .stream_content_chronologically(index)
        .boxed_local()
        .take(50);

    while let Some(result) = stream.next().await {
        match result {
            Ok(cid) => callback.emit(cid),
            Err(e) => {
                error!(&format!("{:#?}", e));
                return;
            }
        }
    }
}
