#!/usr/bin/env bash
rustup install nightly;
grep -r -l 'malloc.h' ffi-deps/FourQlib/FourQ_32bit/schnorrq.c | sort | uniq | xargs perl -e "s/malloc.h/stdlib.h/" -pi
cargo +nightly build;
