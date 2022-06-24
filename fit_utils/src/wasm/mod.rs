use wasm_bindgen::prelude::wasm_bindgen;

pub mod wasm_decoder;
pub use wasm_decoder::WasmDecoder;

#[wasm_bindgen]
pub fn setup() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    wasm_logger::init(wasm_logger::Config::new(log::Level::Info));
}
