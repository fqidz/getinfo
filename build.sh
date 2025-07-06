#!/usr/bin/env bash

# https://github.com/johnthagen/min-sized-rust
RUSTFLAGS="-C target-cpu=native -C link-arg=-fuse-ld=lld" \
    cargo build --release && \
    upx --ultra-brute --lzma ./target/release/getinfo
