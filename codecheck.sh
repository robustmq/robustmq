#!/bin/sh
cargo clippy --all -- --deny warnings
cargo clippy --fix --allow-dirty
cargo fix --allow-dirty
cargo +nightly fmt --all