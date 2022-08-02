use defluencer::Defluencer;
use ipfs_api::{IpfsService, DEFAULT_URI};

use linked_data::types::PeerId;

use gloo_console::error;

use gloo_storage::{LocalStorage, Storage};

use serde::Serialize;

#[derive(Clone)]
pub struct IPFSContext {
    pub client: IpfsService,
    pub peer_id: PeerId,
    pub defluencer: Defluencer,
}

impl PartialEq for IPFSContext {
    fn eq(&self, other: &Self) -> bool {
        self.peer_id == other.peer_id
    }
}

impl IPFSContext {
    pub async fn new(url: Option<String>) -> Option<Self> {
        let url = match &url {
            Some(url) => url,
            None => DEFAULT_URI,
        };

        let client = match IpfsService::new(url) {
            Ok(client) => client,
            Err(e) => {
                error!(&format!("{:#?}", e));

                return None;
            }
        };

        let peer_id = match client.peer_id().await {
            Ok(peer_id) => peer_id.into(),
            Err(e) => {
                error!(&format!("{:#?}", e));

                return None;
            }
        };

        let defluencer = Defluencer::new(client.clone());

        Some(Self {
            client,
            peer_id,
            defluencer,
        })
    }
}

const IPFS_API_ADDRS_KEY: &str = "ipfs_api_addrs";

/// Return IPFS api url from storage or default.
pub fn get_ipfs_addr() -> Result<String, String> {
    match LocalStorage::get(IPFS_API_ADDRS_KEY) {
        Ok(url) => Ok(url),
        Err(_) => Err(DEFAULT_URI.to_owned()),
    }
}

/// Save IPFS api url to local storage.
pub fn set_ipfs_addr<T>(msg: T)
where
    T: Serialize,
{
    if let Err(e) = LocalStorage::set(IPFS_API_ADDRS_KEY, &msg) {
        error!(&format!("{:?}", e));
    }
}
