#!/bin/bash

set -o errexit
set -o nounset
set -o xtrace


rustup target add $TARGET
cargo test --target $TARGET
cargo build --release --target $TARGET

