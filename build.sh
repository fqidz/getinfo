#!/usr/bin/env bash

# https://github.com/johnthagen/min-sized-rust
RUSTFLAGS="-C target-cpu=native -C link-arg=-fuse-ld=lld" \
    cargo build --release && \
    upx --best --lzma ./target/release/getinfo
