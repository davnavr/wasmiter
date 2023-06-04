#!/usr/bin/env -S just --justfile
# Shebang should be compatible with Git Bash for Windows

default: (clippy_all)

list:
    @just --list --justfile {{justfile()}}

cfg_nostd := "--no-default-features"
cfg_alloc := "--no-default-features --features alloc"

test_alloc $RUST_BACKTRACE="1":
    cargo nextest run {{cfg_alloc}}

test_full $RUST_BACKTRACE="1":
    cargo nextest run

test_all $RUST_BACKTRACE="1": (test_full RUST_BACKTRACE) (test_alloc RUST_BACKTRACE)

clippy_nostd:
    cargo clippy {{cfg_nostd}}

clippy_alloc:
    cargo clippy {{cfg_alloc}}

clippy_all: (clippy_nostd) (clippy_alloc)
    cargo clippy
