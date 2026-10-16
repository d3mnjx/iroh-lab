#!/usr/bin/env bash
set -euo pipefail
version=0.2.122
platform=x86_64-unknown-linux-musl
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
curl -fLsS --retry 3 "https://github.com/wasm-bindgen/wasm-bindgen/releases/download/${version}/wasm-bindgen-${version}-${platform}.tar.gz" -o "$work/archive.tar.gz"
tar -xzf "$work/archive.tar.gz" -C "$work"
install "$work/wasm-bindgen-${version}-${platform}/wasm-bindgen" "$HOME/.cargo/bin/wasm-bindgen"
wasm-bindgen --version
