// The worker has its own scope and no direct access to functions/objects of the
// global scope. We import the generated JS file to make `wasm_bindgen`
// available which we need to initialize our WASM code.
importScripts('./pkg/fit_utils.js');

console.log('Initializing worker')

// In the worker, we have a different struct that we want to use as in
// `index.js`.
const { setup, WasmDecoder } = wasm_bindgen;

async function init_wasm_in_worker() {
  // Load the wasm file by awaiting the Promise returned by `wasm_bindgen`.n
  await wasm_bindgen('./pkg/fit_utils_bg.wasm');

  setup();

  console.log('Initialized worker')

  let decoder = WasmDecoder.new();
  self.onmessage = async event => {
    decoder.process(event.data, msg => self.postMessage(msg));
  };
};

init_wasm_in_worker();
