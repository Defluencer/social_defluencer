#![cfg(target_arch = "wasm32")]

use std::collections::HashSet;

use futures_util::{StreamExt, TryStreamExt};

use linked_data::identity::Identity;

use utils::ipfs::IPFSContext;

use wasm_bindgen_futures::spawn_local;

use ybc::{Container, Section};

use yew::prelude::*;

use cid::Cid;

use gloo_console::{error, info};

use crate::comment::Comment;

use ipfs_api::IpfsService;

use defluencer::Defluencer;

const MAX_CRAWL_RESULT: usize = 50;

#[derive(Properties, PartialEq)]
pub struct Props {
    /// Media Content Cid
    pub cid: Cid,
}

pub struct Comments {
    comments_set: HashSet<Cid>,
}

pub enum Msg {
    Comment(Cid),
}

impl Component for Comments {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        #[cfg(debug_assertions)]
        info!("Content Page Create");

        let (context, _) = ctx
            .link()
            .context::<IPFSContext>(Callback::noop())
            .expect("IPFS Context");

        spawn_local(get_comment_cids(
            ctx.link().callback(Msg::Comment),
            context.client.clone(),
            ctx.props().cid,
        ));

        Self {
            comments_set: HashSet::default(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Comment(cid) => self.comments_set.insert(cid),
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        #[cfg(debug_assertions)]
        info!("Content Page View");

        html! {
        <Section>
            <Container>
            {
                self.comments_set.iter().map(|cid| {
                    let cid = *cid;

                    html! {
                        <Comment {cid} />
                    }
                }).collect::<Html>()
            }
            </Container>
        </Section>
        }
    }
}

async fn get_comment_cids(callback: Callback<Cid>, ipfs: IpfsService, cid: Cid) {
    // What context should be used for comments?
    // Could load channel of creator of content first then local follow list

    let defluencer = Defluencer::new(ipfs.clone());

    let identity: Identity = match ipfs.dag_get(cid, Some("/identity")).await {
        Ok(id) => id,
        Err(e) => {
            error!(&format!("{:#?}", e));
            return;
        }
    };

    let mut addresses = HashSet::new();

    if let Some(ipns) = identity.channel_ipns {
        addresses.insert(ipns);
    }

    //TODO add addresses from local follow list

    defluencer
        .streaming_web_crawl(addresses.into_iter())
        .take(MAX_CRAWL_RESULT)
        .map_ok(|(_, metadata)| metadata.comment_index)
        .try_filter_map(|option| async move {
            match option {
                Some(ipld) => Ok(Some(ipld)),
                None => Ok(None),
            }
        })
        .map_ok(|link| defluencer.stream_comments(link, cid))
        .try_flatten()
        .for_each(|result| async {
            match result {
                Ok(cid) => callback.emit(cid),
                Err(e) => error!(&format!("{:#?}", e)),
            }
        })
        .await;
}
