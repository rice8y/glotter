#!/usr/bin/env bash
set -euo pipefail

mkdir -p wasm-plugin/model
curl -L -o wasm-plugin/model/lid.176.ftz https://dl.fbaipublicfiles.com/fasttext/supervised-models/lid.176.ftz
