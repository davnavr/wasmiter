#!/usr/bin/env -S just --justfile
# Shebang should be compatible with Git Bash for Windows

check: (clippy_all) (test_all)

list:
    @just --list --justfile {{justfile()}}

fmt:
    cargo fmt

cfg_nostd := "--no-default-features"
cfg_alloc := "--no-default-features --features alloc"

# Test

test_nostd $RUST_BACKTRACE="1":
    cargo nextest run {{cfg_nostd}}

test_alloc $RUST_BACKTRACE="1":
    cargo nextest run {{cfg_alloc}}

test_full $RUST_BACKTRACE="1":
    cargo nextest run

test_all $RUST_BACKTRACE="1": (test_full RUST_BACKTRACE) (test_alloc RUST_BACKTRACE) (test_nostd RUST_BACKTRACE)

# Clippy

clippy_nostd:
    cargo clippy {{cfg_nostd}}

clippy_alloc:
    cargo clippy {{cfg_alloc}}

clippy_full:
    cargo clippy

clippy_all: (clippy_full) (clippy_alloc) (clippy_nostd)

# Fuzzing

# Run cargo-fuzz on the given target; requires a nightly version of Rust.
fuzz target='parser_valid':
    cargo +nightly fuzz run {{target}}
