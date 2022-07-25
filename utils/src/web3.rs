use web3::{
    api::Namespace,
    contract::ens::Ens,
    transports::eip_1193::{Eip1193, Provider},
    Web3,
};

use linked_data::types::Address;

use gloo_console::error;

#[derive(Clone)]
pub struct Web3Context {
    pub client: Web3<Eip1193>,
    pub ens: Ens<Eip1193>,
    pub addr: Address,
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
        let client = Web3::new(transport.clone());
        let ens = Ens::new(transport);

        let addr = match client.eth().request_accounts().await {
            Ok(addresses) => addresses[0].0,
            Err(e) => {
                error!(&format!("{:#?}", e));
                return None;
            }
        };

        Some(Self { client, ens, addr })
    }
}
