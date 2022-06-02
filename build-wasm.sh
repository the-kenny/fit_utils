#!/usr/bin/env bash

set -ex

mkdir -p web/
wasm-pack build --target no-modules --features wasm
cp src/wasm/index.html src/wasm/worker.js pkg/*.js pkg/*.wasm web/