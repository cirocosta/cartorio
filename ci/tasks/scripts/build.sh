#!/bin/bash

set -o errexit
set -o nounset
set -o xtrace

export RUST_BACKTRACE=1

rustup target add $TARGET
cargo test --target $TARGET
cargo build --release --target $TARGET

