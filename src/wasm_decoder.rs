use log::info;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::decoder::{FitDecodeResult, FitDecoder};

#[wasm_bindgen]
pub struct WasmFitDecoder(FitDecoder);

#[wasm_bindgen]
impl WasmFitDecoder {
    pub fn new() -> Self {
        WasmFitDecoder(FitDecoder::new())
    }

    pub fn process(&mut self, chunk: &[u8]) {
        self.0.add_chunk(chunk);

        let mut n = 0;
        loop {
            match self.0.decode() {
                Ok(FitDecodeResult::Record(msg)) => {
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
