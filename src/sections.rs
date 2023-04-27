use crate::parser::{Input, Parser, Result};

/// A [section *id*](https://webassembly.github.io/spec/core/binary/modules.html#sections)
/// is a byte value that indicates what kind of contents are contained within a WebAssembly
/// [`Section`].
pub type SectionId = std::num::NonZeroU8;

/// Indicates what kind of contents are contained within a WebAssembly [`Section`].
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum SectionKind {
    /// The section is a known value documented in the
    /// [WebAssembly specification](https://webassembly.github.io/spec/core/binary/modules.html#sections)
    Id(SectionId),
    /// The section is a
    /// [custom section](https://webassembly.github.io/spec/core/binary/modules.html#binary-customsec)
    /// with the given name.
    Custom(std::borrow::Cow<'static, str>),
}

/// Represents a
/// [WebAssembly section](https://webassembly.github.io/spec/core/binary/modules.html#sections).
#[derive(Debug)]
pub struct Section<I: Input> {
    kind: SectionKind,
    contents: I,
}

/// Represents the
/// [sequence of sections](https://webassembly.github.io/spec/core/binary/modules.html#binary-module)
/// in a WebAssembly module.
#[derive(Debug)]
pub struct SectionSequence<I: Input> {
    parser: Parser<I>,
}

impl<I: Input> SectionSequence<I> {
    /// Creates a sequence of sections read from the given [`Parser`].
    pub fn new(parser: Parser<I>) -> Self {
        Self { parser }
    }
}
