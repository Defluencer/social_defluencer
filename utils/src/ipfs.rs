use ipfs_api::{IpfsService, DEFAULT_URI};

use linked_data::types::PeerId;

use gloo_console::error;

#[derive(Clone)]
pub struct IPFSContext {
    pub client: IpfsService,
    pub peer_id: PeerId,
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

        Some(Self { client, peer_id })
    }
}
