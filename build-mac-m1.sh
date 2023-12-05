#!/usr/bin/env bash
rustup default stable-aarch64-apple-darwin;
rustup install nightly-aarch64-apple-darwin;
rustup default nightly-aarch64-apple-darwin;
grep -r -l 'malloc.h' ffi-deps/FourQlib/FourQ_32bit/schnorrq.c | sort | uniq | xargs perl -e "s/malloc.h/stdlib.h/" -pi
cargo build;
