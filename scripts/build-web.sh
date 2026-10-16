#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build --locked --release --target wasm32-unknown-unknown --lib
rm -rf dist
mkdir -p dist/wasm
cp -R web/. dist/
wasm-bindgen target/wasm32-unknown-unknown/release/iroh_lab.wasm --out-dir dist/wasm --target web --no-typescript
node scripts/service-worker.mjs
