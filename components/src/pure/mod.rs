#![cfg(target_arch = "wasm32")]

mod content;
mod image;

pub use content::Content;
pub use image::IPFSImage;
