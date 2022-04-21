use log::info;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

use crate::decoder::{FitDecodeResult, FitDecoder};

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
                    callback
                        .call1(
                            &JsValue::undefined(),
                            &JsValue::from_str(&crate::to_json(&msg).unwrap().to_string()),
                        )
                        .unwrap();
                    // info!("{msg:?}");
                    n += 1
                }
                Ok(FitDecodeResult::NotEnoughData) => break,
                Err(_) => todo!(),
            }
        }

        info!("Processed {n} messages");
    }
}
