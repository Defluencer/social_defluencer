#![cfg(target_arch = "wasm32")]

use std::collections::HashSet;

use gloo_storage::{LocalStorage, Storage};

use gloo_console::error;

use linked_data::types::IPLDLink;

const CURRENT_ID_KEY: &str = "current_id";
const ID_LIST_KEY: &str = "id_list";

pub fn get_identities() -> Option<HashSet<IPLDLink>> {
    match LocalStorage::get(ID_LIST_KEY) {
        Ok(list) => return Some(list),
        Err(e) => error!(&format!("{:?}", e)),
    }

    None
}

pub fn set_identities(list: HashSet<IPLDLink>) {
    if let Err(e) = LocalStorage::set(ID_LIST_KEY, list) {
        error!(&format!("{:?}", e));
    }
}

pub fn get_current_identity() -> Option<IPLDLink> {
    match LocalStorage::get(CURRENT_ID_KEY) {
        Ok(id) => return Some(id),
        Err(e) => error!(&format!("{:?}", e)),
    }

    None
}

pub fn set_current_identity(ipld: IPLDLink) {
    if let Err(e) = LocalStorage::set(CURRENT_ID_KEY, ipld) {
        error!(&format!("{:?}", e));
    }
}

pub fn clear_current_identity() {
    LocalStorage::delete(CURRENT_ID_KEY)
}
