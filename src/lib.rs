#![doc = include_str!("../README.md")]
#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(unreachable_pub)]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(clippy::undocumented_unsafe_blocks)]

pub mod parser;

mod sections;

pub use sections::{Section, SectionId, SectionKind, SectionSequence};

/// Reads a [WebAssembly module binary](https://webassembly.github.io/spec/core/binary/index.html),
/// returning the sequence of sections.
pub fn parse_module_sections<I: parser::Input>(binary: I) -> parser::Result<SectionSequence<I>> {
    todo!()
}
