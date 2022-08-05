#![cfg(target_arch = "wasm32")]

mod app;

use app::App;

fn main() {
    yew::Renderer::<App>::new().render();
}
