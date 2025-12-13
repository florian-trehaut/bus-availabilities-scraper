#![recursion_limit = "256"]

#[cfg(target_arch = "wasm32")]
use app::components::App;
#[cfg(target_arch = "wasm32")]
use leptos::mount::hydrate_body;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    #[allow(clippy::expect_used)]
    console_log::init_with_level(log::Level::Debug).expect("error initializing logger");

    hydrate_body(App);
}
