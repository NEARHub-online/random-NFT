#!/bin/bash
set -e
rustup target add wasm32-unknown-unknown
cd "`dirname $0`"
source flags.sh
RUSTFLAGS='-C link-arg=-s' cargo build --all --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/*.wasm ./res/