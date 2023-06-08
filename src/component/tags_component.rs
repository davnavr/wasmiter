use crate::{
    bytes::Bytes,
    parser::{Result, ResultExt},
};

/// Represents the
/// [**tags** component](https://webassembly.github.io/exception-handling/core/syntax/modules.html#tags) of a
/// WebAssembly module, stored in and parsed from the
/// [*tags section*](https://webassembly.github.io/exception-handling/core/binary/modules.html#tag-section).
#[derive(Clone, Copy)]
pub struct TagsComponent<B: Bytes> {}
