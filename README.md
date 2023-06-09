# `wasmiter`

Low-level WebAssembly parser, with a focus on zero allocations.

Designed to allow easier processing of a WebAssembly module's sections in parallel.

## Supported WebAssembly Proposals

The following [WebAssembly proposals](https://github.com/WebAssembly/proposals) are **fully supported** by the parser:
- [Merged (Phase 5) Proposals](https://github.com/WebAssembly/proposals/blob/main/finished-proposals.md):
  - [Mutable Globals](https://github.com/WebAssembly/mutable-global)
  - [Non-Trapping Float-to-Integer Conversions](https://github.com/WebAssembly/nontrapping-float-to-int-conversions)
  - [Sign Extension Operators](https://github.com/WebAssembly/sign-extension-ops)
  - [Multi-Value](https://github.com/WebAssembly/multi-value)
  - [Reference Types](https://github.com/WebAssembly/reference-types)
  - [Bulk Memory Operations](https://github.com/WebAssembly/bulk-memory-operations)
  - [Fixed Width SIMD](https://github.com/webassembly/simd)
- Phase 4 Proposals:
  - [Tail Call](https://github.com/WebAssembly/tail-call)
  - [Extended Constant Expressions](https://github.com/WebAssembly/extended-const)
- Phase 3 Proposals:
  - [Multi-Memory](https://github.com/WebAssembly/multi-memory)
  - [Memory64](https://github.com/WebAssembly/memory64)
  - [Exception Handling](https://github.com/WebAssembly/exception-handling)
  - [Threads](https://github.com/webassembly/threads)

## `#![no_std]` Compatibility

This library is completely `#![no_std]` compatible, capable of being used with or without the
[`alloc`](https://doc.rust-lang.org/alloc/) library. The `std` and `alloc` features control this.
