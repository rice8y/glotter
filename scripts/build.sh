#!/usr/bin/env bash
set -euo pipefail

if [ ! -f wasm-plugin/model/lid.176.ftz ]; then
  echo "wasm-plugin/model/lid.176.ftz is missing. Run ./scripts/fetch-model.sh first." >&2
  exit 1
fi

rustup target add wasm32-unknown-unknown

cargo build \
  --manifest-path wasm-plugin/Cargo.toml \
  --release \
  --target wasm32-unknown-unknown \
  --target-dir target

wasm-opt -Oz \
  --strip-debug \
  --strip-producers \
  -o package/glotter.wasm \
  target/wasm32-unknown-unknown/release/glotter_plugin.wasm