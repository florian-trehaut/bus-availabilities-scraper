#![recursion_limit = "256"]

use app::components::App;
use leptos::mount::hydrate_body;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    #[allow(clippy::expect_used)]
    console_log::init_with_level(log::Level::Debug).expect("error initializing logger");

    hydrate_body(App);
}
