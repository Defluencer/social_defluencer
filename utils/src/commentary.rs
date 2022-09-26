#![cfg(target_arch = "wasm32")]

use cid::Cid;

use linked_data::comments::Comment;
use yew::Callback;

#[derive(Clone)]
pub struct CommentaryContext {
    pub callback: Callback<(Cid, Comment)>,
}

impl PartialEq for CommentaryContext {
    fn eq(&self, other: &Self) -> bool {
        self.callback != other.callback
    }
}

impl CommentaryContext {
    pub async fn new(callback: Callback<(Cid, Comment)>) -> Self {
        Self { callback }
    }
}
