#![cfg(target_arch = "wasm32")]

use cid::Cid;

use defluencer::{
    channel::{local::LocalUpdater, Channel},
    crypto::signers::MetamaskSigner,
    user::User,
};

use ipfs_api::IpfsService;

use linked_data::types::IPNSAddress;

#[derive(Clone, PartialEq)]
pub struct UserContext {
    pub user: User<MetamaskSigner>,
}

impl UserContext {
    pub fn new(ipfs: IpfsService, signer: MetamaskSigner, identity: Cid) -> Self {
        let user = User::new(ipfs, signer, identity);

        Self { user }
    }
}

#[derive(Clone, PartialEq)]
pub struct ChannelContext {
    pub channel: Channel<LocalUpdater>,
}

impl ChannelContext {
    pub fn new(ipfs: IpfsService, key: String, addr: IPNSAddress) -> Self {
        let updater = LocalUpdater::new(ipfs.clone(), key);
        let channel = Channel::new(ipfs, addr, updater);

        Self { channel }
    }
}
