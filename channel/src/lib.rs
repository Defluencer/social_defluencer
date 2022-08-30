#![cfg(target_arch = "wasm32")]

mod manage_content;

use manage_content::ManageContent;

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

use gloo_console::{error, info};

use ipfs_api::IpfsService;

use linked_data::{
    channel::ChannelMetadata,
    types::{IPLDLink, IPNSAddress},
};

use utils::{
    defluencer::{ChannelContext, UserContext},
    ipfs::IPFSContext,
};

use ybc::{Container, Section};

use yew::{platform::spawn_local, prelude::*};

use cid::Cid;

#[derive(Properties, PartialEq)]
pub struct Props {
    /// Channel Address
    pub cid: Cid,
}

//TODO If live, display video

/// social.defluencer.eth/#/channel/<IPNS_HERE>
///
/// A specific channel page
pub struct ChannelPage {
    handle: AbortHandle,

    metadata: Option<ChannelMetadata>,

    content: VecDeque<Cid>,
    content_cb: Callback<Cid>,
    create_cb: Callback<Cid>,

    other_remove_cb: Callback<Cid>,

    channel: bool,
}

pub enum Msg {
    Update(ChannelMetadata),
    Content(Cid),
    Created(Cid),
    Remove(Cid),
}

impl Component for ChannelPage {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        #[cfg(debug_assertions)]
        info!("Channel Page Create");

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

        let content_cb = ctx.link().callback(Msg::Content);
        let create_cb = ctx.link().callback(Msg::Created);

        let mut channel = false;

        if ctx
            .link()
            .context::<UserContext>(Callback::noop())
            .is_some()
        {
            if let Some((context, _)) = ctx.link().context::<ChannelContext>(Callback::noop()) {
                if context.channel.get_address() == ctx.props().cid.into() {
                    channel = true;
                }
            }
        }

        let other_remove_cb = ctx.link().callback(Msg::Remove);

        Self {
            handle,

            metadata: None,

            content: Default::default(),
            content_cb,
            create_cb,

            other_remove_cb,

            channel,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        #[cfg(debug_assertions)]
        info!("Channel Page Update");

        match msg {
            Msg::Update(metadata) => self.on_channel_update(ctx, metadata),
            Msg::Content(cid) => self.on_older_content(cid),
            Msg::Created(cid) => self.on_newer_content(cid),
            Msg::Remove(cid) => self.on_remove_content(cid),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        #[cfg(debug_assertions)]
        info!("Channel Page View");

        match &self.metadata {
            Some(meta) => self.render_channel(ctx, meta),
            None => self.render_no_channel(),
        }
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        #[cfg(debug_assertions)]
        info!("Channel Page Destroy");

        self.handle.abort();
    }
}

impl ChannelPage {
    fn render_channel(&self, ctx: &Context<ChannelPage>, meta: &ChannelMetadata) -> Html {
        html! {
        <>
            <NavigationBar />
            <Section>
                <Identification cid={meta.identity.link} />
                <Container>
                    if self.channel
                    {
                        <ManageContent cid={ctx.props().cid} content_cb={self.create_cb.clone()} remove_cb={self.other_remove_cb.clone()} />
                    }
                    {
                        self.content.iter().rev().map(|&cid| {
                            html! {
                                <Thumbnail key={cid.to_string()} {cid} />
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
        <>
            <NavigationBar />
            <Searching />
        </>
        }
    }

    fn on_channel_update(&mut self, ctx: &Context<Self>, metadata: ChannelMetadata) -> bool {
        if let Some(index) = metadata.content_index {
            self.content.clear();

            let (context, _) = ctx
                .link()
                .context::<IPFSContext>(Callback::noop())
                .expect("IPFS Context");

            spawn_local(stream_content(
                context.client.clone(),
                self.content_cb.clone(),
                index,
            ));
        }

        self.metadata = Some(metadata);

        true
    }

    fn on_older_content(&mut self, cid: Cid) -> bool {
        if self.content.len() < 50 {
            self.content.push_front(cid);

            return true;
        }

        false
    }

    fn on_newer_content(&mut self, cid: Cid) -> bool {
        self.content.push_back(cid);

        if self.content.len() > 50 {
            self.content.pop_front();
        }

        true
    }

    fn on_remove_content(&mut self, cid: Cid) -> bool {
        if let Some(index) = self.content.iter().position(|&item| item == cid) {
            self.content.remove(index);

            return true;
        }

        false
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
