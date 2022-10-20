#![cfg(target_arch = "wasm32")]

use std::collections::HashSet;

use cid::Cid;

use gloo_console::error;

use ipfs_api::IpfsService;

use linked_data::{
    channel::ChannelMetadata,
    media::Media,
    types::{IPLDLink, IPNSAddress},
};

use serde::de::DeserializeOwned;

use yew::Callback;

use defluencer::Defluencer;

use futures_util::stream::{
    AbortRegistration, Abortable, FuturesUnordered, StreamExt, TryStreamExt,
};

/// Resolve multiple IPNS addresses then get the channel metadata.
pub async fn get_channels(
    ipfs: IpfsService,
    callback: Callback<(IPNSAddress, Cid, ChannelMetadata)>,
    addresses: HashSet<IPNSAddress>,
) {
    let count = addresses.len();

    let update_pool: FuturesUnordered<_> = addresses
        .into_iter()
        .map(|addr| {
            let ipfs = ipfs.clone();

            async move {
                match ipfs.name_resolve(addr.into()).await {
                    Ok(cid) => Ok((addr, cid)),
                    Err(e) => Err(e),
                }
            }
        })
        .collect();

    let mut stream = update_pool
        .map_ok(|(addr, cid)| {
            let ipfs = ipfs.clone();

            async move {
                match ipfs.dag_get::<&str, ChannelMetadata>(cid, None).await {
                    Ok(dag) => Ok((addr, cid, dag)),
                    Err(e) => Err(e),
                }
            }
        })
        .try_buffer_unordered(count);

    while let Some(result) = stream.next().await {
        match result {
            Ok(tuple) => callback.emit(tuple),
            Err(e) => error!(&format!("{:#?}", e)),
        }
    }
}

/// Subscribe and get latest channel metadata
pub async fn channel_subscribe(
    ipfs: IpfsService,
    callback: Callback<(IPNSAddress, Cid, ChannelMetadata)>,
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
                        Ok(dag) => Ok((addr, cid, dag)),
                        Err(e) => Err(e.into()),
                    },
                    Err(e) => Err(e),
                }
            }
        })
        .buffer_unordered(2);

    let stream = Abortable::new(stream, regis);

    futures_util::pin_mut!(stream);

    while let Some(result) = stream.next().await {
        match result {
            Ok(tuple) => callback.emit(tuple),
            Err(e) => error!(&format!("{:#?}", e)),
        }
    }
}

/// Stream all content of a channel
pub async fn stream_content(
    ipfs: IpfsService,
    callback: Callback<(Cid, Media)>,
    index: IPLDLink,
    regis: AbortRegistration,
) {
    let defluencer = Defluencer::new(ipfs.clone());

    let stream = defluencer
        .stream_content_rev_chrono(index)
        .map_ok(|cid| {
            let ipfs = ipfs.clone();

            async move {
                match ipfs.dag_get::<&str, Media>(cid, Some("/link")).await {
                    Ok(dag) => Ok((cid, dag)),
                    Err(e) => Err(e.into()),
                }
            }
        })
        .try_buffer_unordered(10);

    let stream = Abortable::new(stream, regis);

    futures_util::pin_mut!(stream);

    while let Some(result) = stream.next().await {
        match result {
            Ok(tuple) => callback.emit(tuple),
            Err(e) => error!(&format!("{:#?}", e)),
        }
    }
}

pub async fn dag_get<T>(ipfs: IpfsService, cid: Cid, callback: Callback<(Cid, T)>)
where
    T: ?Sized + DeserializeOwned,
{
    match ipfs.dag_get::<&str, T>(cid, None).await {
        Ok(dag) => callback.emit((cid, dag)),
        Err(e) => error!(&format!("{:#?}", e)),
    }
}
