#![cfg(target_arch = "wasm32")]

use std::collections::HashSet;

use gloo_storage::{LocalStorage, Storage};

use gloo_console::error;

use linked_data::types::IPNSAddress;

const FOLLOW_LIST: &str = "follow_list";

/// Get addresses of all followees
pub fn get_follow_list() -> HashSet<IPNSAddress> {
    match LocalStorage::get(FOLLOW_LIST) {
        Ok(list) => return list,
        Err(e) => error!(&format!("{:?}", e)),
    }

    HashSet::default()
}

/// Set new list of followees
pub fn set_follow_list(list: HashSet<IPNSAddress>) {
    if let Err(e) = LocalStorage::set(FOLLOW_LIST, list) {
        error!(&format!("{:?}", e));
    }
}
