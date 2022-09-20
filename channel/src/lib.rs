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
    stream::{AbortHandle, AbortRegistration, Abortable},
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

use ybc::{Box, Button, Container, Section};

use yew::{platform::spawn_local, prelude::*};

use cid::Cid;

#[derive(Properties, PartialEq)]
pub struct Props {
    /// Channel Address
    pub addr: Cid,
}

//TODO If live, display video

//TODO display all comments too

/// social.defluencer.eth/#/channel/<IPNS_HERE>
///
/// A specific channel page
pub struct ChannelPage {
    sub_handle: AbortHandle,
    stream_handle: Option<AbortHandle>,

    metadata: Option<ChannelMetadata>,

    content: VecDeque<Cid>,
    content_cb: Callback<Cid>,

    own_channel: bool,

    follow_cb: Callback<MouseEvent>,
    following: bool,
}

pub enum Msg {
    Update(ChannelMetadata),
    Content(Cid),
    Follow,
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

        spawn_local(get_channel(
            context.client.clone(),
            ctx.link().callback(Msg::Update),
            ctx.props().addr.into(),
        ));

        let (sub_handle, regis) = AbortHandle::new_pair();

        spawn_local(channel_subscribe(
            context.client.clone(),
            ctx.link().callback(Msg::Update),
            ctx.props().addr.into(),
            regis,
        ));

        let content_cb = ctx.link().callback(Msg::Content);

        let mut channel = false;

        if ctx
            .link()
            .context::<UserContext>(Callback::noop())
            .is_some()
        {
            if let Some((context, _)) = ctx.link().context::<ChannelContext>(Callback::noop()) {
                if context.channel.get_address() == ctx.props().addr.into() {
                    channel = true;
                }
            }
        }

        let follow_cb = ctx.link().callback(|_| Msg::Follow);

        let following = {
            let list = utils::follows::get_follow_list();

            list.contains(&ctx.props().addr.into())
        };

        Self {
            sub_handle,
            stream_handle: None,

            metadata: None,

            content: Default::default(),
            content_cb,

            own_channel: channel,

            follow_cb,
            following,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        #[cfg(debug_assertions)]
        info!("Channel Page Update");

        match msg {
            Msg::Update(metadata) => self.on_channel_update(ctx, metadata),
            Msg::Content(cid) => self.on_content(cid),
            Msg::Follow => self.on_follow(ctx),
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

        self.sub_handle.abort();

        if let Some(handle) = self.stream_handle.take() {
            handle.abort();
        }
    }
}

impl ChannelPage {
    fn render_channel(&self, ctx: &Context<ChannelPage>, meta: &ChannelMetadata) -> Html {
        html! {
        <>
            <NavigationBar />
            <Section>
                <Identification cid={meta.identity.link} />
                <Button onclick={self.follow_cb.clone()} >
                {
                    if self.following {"Unfollow"} else {"Follow"}
                }
                </Button>
                <Container>
                    if self.own_channel
                    {
                        <ManageContent addr={ctx.props().addr} />
                    }
                    {
                        self.content.iter().rev().map(|&cid| {
                            html! {
                            <Box>
                                <Thumbnail {cid} />
                            </Box>
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
        if self.metadata.as_ref() == Some(&metadata) {
            return false;
        }

        if let Some(index) = metadata.content_index {
            if let Some(handle) = self.stream_handle.take() {
                handle.abort();
            }

            self.content.clear();

            let (context, _) = ctx
                .link()
                .context::<IPFSContext>(Callback::noop())
                .expect("IPFS Context");

            let (stream_handle, regis) = AbortHandle::new_pair();

            spawn_local(stream_content(
                context.client.clone(),
                self.content_cb.clone(),
                index,
                regis,
            ));

            self.stream_handle = Some(stream_handle);
        }

        self.metadata = Some(metadata);

        true
    }

    fn on_content(&mut self, cid: Cid) -> bool {
        if self.content.len() < 50 {
            self.content.push_front(cid);

            return true;
        }

        false
    }

    fn on_follow(&mut self, ctx: &Context<Self>) -> bool {
        let mut list = utils::follows::get_follow_list();

        if !self.following {
            list.insert(ctx.props().addr.into());
        } else {
            list.remove(&ctx.props().addr.into());
        }

        utils::follows::set_follow_list(list);

        self.following = !self.following;

        true
    }
}

async fn get_channel(ipfs: IpfsService, callback: Callback<ChannelMetadata>, addr: IPNSAddress) {
    let cid = match ipfs.name_resolve(addr.into()).await {
        Ok(cid) => cid,
        Err(e) => {
            error!(&format!("{:#?}", e));
            return;
        }
    };

    match ipfs.dag_get::<&str, ChannelMetadata>(cid, None).await {
        Ok(dag) => callback.emit(dag),
        Err(e) => error!(&format!("{:#?}", e)),
    }
}

async fn channel_subscribe(
    ipfs: IpfsService,
    callback: Callback<ChannelMetadata>,
    addr: IPNSAddress,
    regis: AbortRegistration,
) {
    let defluencer = Defluencer::new(ipfs.clone());

    let stream = defluencer
        .subscribe_channel_updates(addr)
        .map(|result| {
            let ipfs = ipfs.clone();

            async move {
                match result {
                    Ok(cid) => match ipfs.dag_get::<&str, ChannelMetadata>(cid, None).await {
                        Ok(dag) => Ok(dag),
                        Err(e) => Err(e.into()),
                    },
                    Err(e) => Err(e),
                }
            }
        })
        .buffer_unordered(2);

    let mut stream = Abortable::new(stream, regis).boxed_local();

    while let Some(result) = stream.next().await {
        match result {
            Ok(dag) => callback.emit(dag),
            Err(e) => {
                error!(&format!("{:#?}", e));
                continue;
            }
        }
    }
}

async fn stream_content(
    ipfs: IpfsService,
    callback: Callback<Cid>,
    index: IPLDLink,
    regis: AbortRegistration,
) {
    let defluencer = Defluencer::new(ipfs.clone());

    let stream = defluencer.stream_content_rev_chrono(index).take(50);

    let mut stream = Abortable::new(stream, regis).boxed_local();

    while let Some(result) = stream.next().await {
        match result {
            Ok(cid) => callback.emit(cid),
            Err(e) => {
                error!(&format!("{:#?}", e));
                continue;
            }
        }
    }
}
