# `wasmiter`

Low-level WebAssembly parser, with a focus on support for processing sections in parallel.

# `#![no_std]` Compatibility

This library is completely `#![no_std]` compatible, capable of being used with or without the
[`alloc`](https://doc.rust-lang.org/alloc/) library. The `std` and `alloc` features control this.
