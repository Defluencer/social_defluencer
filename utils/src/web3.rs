use defluencer::signatures::ethereum::EthereumSigner;
use web3::{
    api::Namespace,
    contract::ens::Ens,
    transports::eip_1193::{Eip1193, Provider},
    Web3,
};

use linked_data::types::Address;

use gloo_console::error;

use gloo_storage::{LocalStorage, Storage};

use serde::Serialize;

#[derive(Clone)]
pub struct Web3Context {
    pub client: Web3<Eip1193>,
    pub ens: Ens<Eip1193>,
    pub addr: Address,
    pub signer: EthereumSigner,
}

impl PartialEq for Web3Context {
    fn eq(&self, other: &Self) -> bool {
        self.addr == other.addr
    }
}

impl Web3Context {
    pub async fn new() -> Option<Self> {
        let provider = match Provider::default() {
            Ok(provider) => match provider {
                Some(prov) => prov,
                None => return None,
            },
            Err(e) => {
                error!(&format!("{:#?}", e));
                return None;
            }
        };

        let transport = Eip1193::new(provider);

        let ens = Ens::new(transport.clone());
        let client = Web3::new(transport);

        let addr = match client.eth().request_accounts().await {
            Ok(addresses) => addresses[0].0,
            Err(e) => {
                error!(&format!("{:#?}", e));
                return None;
            }
        };

        let signer = EthereumSigner::new(addr, client.clone());

        Some(Self {
            client,
            ens,
            addr,
            signer,
        })
    }
}

const WALLET_ADDRS_KEY: &str = "wallet_addrs";

/// Return wallet address from local storage if possible.
pub fn get_wallet_addr() -> Option<String> {
    LocalStorage::get(WALLET_ADDRS_KEY).ok()
}

/// Save wallet address to local storage.
pub fn set_wallet_addr<T>(msg: T)
where
    T: Serialize,
{
    if let Err(e) = LocalStorage::set(WALLET_ADDRS_KEY, &msg) {
        error!(&format!("{:?}", e));
    }
}
