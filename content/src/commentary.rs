#![cfg(target_arch = "wasm32")]

use std::collections::HashSet;

use futures_util::{stream::FuturesUnordered, StreamExt, TryStreamExt};

use linked_data::{
    identity::Identity,
    types::{IPLDLink, IPNSAddress},
};

use utils::{follows::get_follow_list, ipfs::IPFSContext};

use ybc::{Container, Section};

use yew::{platform::spawn_local, prelude::*};

use cid::Cid;

use gloo_console::{error, info};

use crate::comment::Comment;

use ipfs_api::IpfsService;

use defluencer::Defluencer;

const MAX_CRAWL_RESULT: usize = 50;

#[derive(Properties, PartialEq)]
pub struct Props {
    /// signed link to Media Content Cid
    pub cid: Cid,
}

pub struct Commentary {
    ipfs: IpfsService,

    comments_set: HashSet<Cid>,
}

pub enum Msg {
    Comment(Cid),
    Follows(Vec<Identity>),
}

impl Component for Commentary {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        #[cfg(debug_assertions)]
        info!("Commentary Create");

        let (context, _) = ctx
            .link()
            .context::<IPFSContext>(Callback::noop())
            .expect("IPFS Context");

        let ipfs = context.client;

        let follows = get_follow_list();

        ctx.link()
            .send_future(get_local_follows(ipfs.clone(), ctx.props().cid, follows));

        Self {
            ipfs,
            comments_set: HashSet::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        #[cfg(debug_assertions)]
        info!("Commentary Update");

        match msg {
            Msg::Comment(cid) => self.comments_set.insert(cid),
            Msg::Follows(vec) => {
                let addresses = vec
                    .into_iter()
                    .filter_map(|item| item.channel_ipns)
                    .collect();

                spawn_local(get_comment_cids(
                    ctx.link().callback(Msg::Comment),
                    self.ipfs.clone(),
                    ctx.props().cid,
                    addresses,
                ));

                false
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        #[cfg(debug_assertions)]
        info!("Commentary View");

        let comment_list = self
            .comments_set
            .iter()
            .map(|cid| {
                let cid = *cid;

                html! {
                    <Comment {cid} />
                }
            })
            .collect::<Html>();

        html! {
        <Section>
            <Container>
                { comment_list }
            </Container>
        </Section>
        }
    }
}

async fn get_local_follows(ipfs: IpfsService, cid: Cid, follows: HashSet<IPLDLink>) -> Msg {
    let pool: FuturesUnordered<_> = follows
        .into_iter()
        .map(|ipld| ipfs.dag_get::<&str, Identity>(ipld.link, Some("/link")))
        .collect();

    pool.push(ipfs.dag_get::<&str, Identity>(cid, Some("/link/identity")));

    let vec = pool
        .filter_map(|result| async move {
            match result {
                Ok(id) => Some(id),
                Err(e) => {
                    error!(&format!("{:#?}", e));
                    None
                }
            }
        })
        .collect()
        .await;

    Msg::Follows(vec)
}

async fn get_comment_cids(
    callback: Callback<Cid>,
    ipfs: IpfsService,
    cid: Cid,
    addresses: HashSet<IPNSAddress>,
) {
    let defluencer = Defluencer::new(ipfs);

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
