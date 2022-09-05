#![cfg(target_arch = "wasm32")]

use std::collections::HashSet;

use futures_util::{StreamExt, TryStreamExt};

use linked_data::types::IPNSAddress;

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
    comments_set: HashSet<Cid>,
}

pub enum Msg {
    Comment(Cid),
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

        let follows = get_follow_list();

        spawn_local(stream_comments(
            context.client,
            ctx.props().cid,
            follows,
            ctx.link().callback(Msg::Comment),
        ));

        Self {
            comments_set: HashSet::default(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        #[cfg(debug_assertions)]
        info!("Commentary Update");

        match msg {
            Msg::Comment(cid) => self.comments_set.insert(cid),
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

async fn stream_comments(
    ipfs: IpfsService,
    cid: Cid,
    follows: HashSet<IPNSAddress>,
    callback: Callback<Cid>,
) {
    let defluencer = Defluencer::new(ipfs);

    defluencer
        .streaming_web_crawl(follows.into_iter())
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
