//! Types and functions for parsing the contents of
//! [WebAssembly custom sections](https://webassembly.github.io/spec/core/appendix/custom.html).

use crate::{bytes::{self, Bytes}, sections, parser::name::Name};

/// Represents a well-known
/// [custom section](https://webassembly.github.io/spec/core/appendix/custom.html) in a WebAssembly
/// module.
#[derive(Clone, Copy)]
#[non_exhaustive]
pub enum CustomSection<B: Bytes> {
}
