use log::info;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

use crate::fit_decoder::{FitDecodeResult, FitDecoder};

#[wasm_bindgen]
pub struct WasmDecoder(FitDecoder);

#[wasm_bindgen]
impl WasmDecoder {
    pub fn new() -> Self {
        WasmDecoder(FitDecoder::new())
    }

    pub fn process(&mut self, chunk: &[u8], callback: &js_sys::Function) {
        self.0.add_chunk(chunk);

        let mut n = 0;
        loop {
            match self.0.poll() {
                Ok(FitDecodeResult::Record(msg)) => {
                    let json = JsValue::from_serde(&crate::to_json(&msg).unwrap()).unwrap();
                    callback.call1(&JsValue::undefined(), &json).unwrap();
                    n += 1
                }
                Ok(FitDecodeResult::NotEnoughData) => break,
                Err(_) => todo!(),
            }
        }

        info!("WasmDecoder.process: Processed {n} messages");
    }
}
