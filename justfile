#!/usr/bin/env -S just --justfile
# Shebang should be compatible with Git Bash for Windows

cfg_nostd := "--no-default-features"
cfg_alloc := "--no-default-features --features alloc"

alias c := clippy_full
alias d := doc
alias f := fmt_all
alias t := test_full

# Runs clippy, followed by all tests
check: clippy_all test_all

# Lists all available recipes
list:
    @just --list --justfile {{justfile()}}

# Runs fmt, by default for all code
fmt *FLAGS="--all":
    cargo fmt {{FLAGS}}

# Runs fmt on the fuzzing code
fmt_fuzz:
    cd ./fuzz/ && cargo fmt

# Runs fmt on all code
fmt_all: fmt fmt_fuzz

# Runs clippy on the full no_std variant of wasmiter
clippy_nostd:
    cargo clippy {{cfg_nostd}}

# Runs clippy on the no_std+alloc variant of wasmiter
clippy_alloc:
    cargo clippy {{cfg_alloc}}

# Runs clippy on the default variant of wasmiter
clippy_full:
    cargo clippy

# Runs clippy on the fuzzing code
clippy_fuzz:
    cd ./fuzz/ && cargo clippy

# Runs clippy on all 3 major variants of wasmiter and the fuzzing code
clippy_all: clippy_full clippy_alloc clippy_nostd clippy_fuzz

# Runs all tests for the full no_std variant of wasmiter
test_nostd $RUST_BACKTRACE="1":
    cargo nextest run {{cfg_nostd}}
    cargo test --doc {{cfg_nostd}}

# Runs all tests for the no_std+alloc variant of wasmiter
test_alloc $RUST_BACKTRACE="1":
    cargo nextest run {{cfg_alloc}}
    cargo test --doc {{cfg_alloc}}

# Runs all tests for the default variant of wasmiter
test_full $RUST_BACKTRACE="1":
    cargo nextest run --all-targets
    cargo test --doc

# Runs all tests for all 3 major variants of wasmiter
test_all $RUST_BACKTRACE="1": (test_full RUST_BACKTRACE) (test_alloc RUST_BACKTRACE) (test_nostd RUST_BACKTRACE)

# Invoke rustdoc, requires a nightly version of Rust
doc *FLAGS='--open':
    RUSTDOCFLAGS="--cfg doc_cfg" cargo +nightly doc {{FLAGS}}

# Runs cargo-fuzz on the given target; requires a nightly version of Rust
fuzz target='parser_random' *FLAGS='-- -jobs=2':
    cargo +nightly fuzz run {{target}} {{FLAGS}}
