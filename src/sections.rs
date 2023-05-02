use crate::parser::{input::Input, Parser, Result, ResultExt};

mod section_kind;

pub use section_kind::{CustomSectionName, SectionId, SectionKind};

/// Represents a
/// [WebAssembly section](https://webassembly.github.io/spec/core/binary/modules.html#sections).
#[derive(Debug)]
pub struct Section<I: Input> {
    kind: SectionKind,
    contents: I,
}

impl<I: Input> Section<I> {
    /// Gets the
    /// [*id* or custom section name](https://webassembly.github.io/spec/core/binary/modules.html#sections)
    /// for this section.
    pub fn kind(&self) -> &SectionKind {
        &self.kind
    }
}

/// Represents the
/// [sequence of sections](https://webassembly.github.io/spec/core/binary/modules.html#binary-module)
/// in a WebAssembly module.
#[derive(Debug)]
pub struct SectionSequence<I: Input> {
    parser: Parser<I>,
}

impl<I: Input> SectionSequence<I> {
    /// Creates a sequence of sections read from the given [`Input`].
    pub fn new(input: I) -> Self {
        Self {
            parser: Parser::new(input),
        }
    }

    fn parse(&mut self) -> Result<Option<Section<I>>> {
        let mut id_byte = 0u8;
        self.parser
            .bytes_exact(std::slice::from_mut(&mut id_byte))
            .context("section id byte")?;

        let kind = SectionId::new(id_byte);

        //let size = self.parser.leb128_u64;

        todo!()
    }
}

impl<I: Input> Iterator for SectionSequence<I> {
    type Item = Result<Section<I>>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.parse().transpose()
    }
}
