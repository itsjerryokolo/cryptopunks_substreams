#!/usr/bin/env sh
set -eu
cargo build --locked --target wasm32-unknown-unknown --release
