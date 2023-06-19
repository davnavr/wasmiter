#!/usr/bin/env -S just --justfile
# Shebang should be compatible with Git Bash for Windows

# Runs clippy, followed by all tests
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
    cargo test --doc {{cfg_nostd}}

test_alloc $RUST_BACKTRACE="1":
    cargo nextest run {{cfg_alloc}}
    cargo test --doc {{cfg_alloc}}

test_full $RUST_BACKTRACE="1":
    cargo nextest run --all-targets
    cargo test --doc

test_all $RUST_BACKTRACE="1": (test_full RUST_BACKTRACE) (test_alloc RUST_BACKTRACE) (test_nostd RUST_BACKTRACE)

# Clippy

# Runs clippy on the full no_std variant of wasmiter
clippy_nostd:
    cargo clippy {{cfg_nostd}}

# Runs clippy on the no_std+alloc variant of wasmiter
clippy_alloc:
    cargo clippy {{cfg_alloc}}

# Runs clippy on the default variant of wasmiter
clippy_full:
    cargo clippy

# Runs clippy on all 3 major variants of wasmiter
clippy_all: (clippy_full) (clippy_alloc) (clippy_nostd)

# Fuzzing

fmt_fuzz:
    cd ./fuzz/ && cargo fmt

clippy_fuzz:
    cd ./fuzz/ && cargo clippy

# Runs cargo-fuzz on the given target; requires a nightly version of Rust
fuzz target='parser_random' *FLAGS='-- -jobs=2':
    cargo +nightly fuzz run {{target}} {{FLAGS}}
