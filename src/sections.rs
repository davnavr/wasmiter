use crate::parser::{input::Input, Parser, Result, ResultExt};

mod section_kind;

pub use section_kind::{CustomSectionName, SectionId, SectionKind};

/// Represents a
/// [WebAssembly section](https://webassembly.github.io/spec/core/binary/modules.html#sections).
#[derive(Debug)]
pub struct Section<I: Input> {
    kind: SectionKind,
    length: u32,
    contents: Parser<I>,
}

impl<I: Input> Section<I> {
    /// Gets the
    /// [*id* or custom section name](https://webassembly.github.io/spec/core/binary/modules.html#sections)
    /// for this section.
    #[inline]
    pub fn kind(&self) -> &SectionKind {
        &self.kind
    }

    /// Gets the length, in bytes, of the content of the section.
    ///
    /// Note that for custom sections, this does **not** include the section name.
    #[inline]
    pub fn length(&self) -> u32 {
        self.length
    }

    /// Consumes the section, returning a [`Parser<I>`] used to read its contents.
    pub fn into_contents(self) -> Parser<I> {
        self.contents
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
    /// Uses the given [`Parser<I>`] to read a sequence of sections.
    pub fn new(parser: Parser<I>) -> Self {
        Self { parser }
    }

    /// Creates a sequence of sections read from the given [`Input`].
    pub fn from_input(input: I) -> Self {
        Self::new(Parser::new(input))
    }

    fn parse(&mut self) -> Result<Option<Section<I::Fork>>> {
        let mut id_byte = 0u8;

        let id_length = self
            .parser
            .bytes(core::slice::from_mut(&mut id_byte))
            .context("section id byte")?;

        if id_length == 0 {
            return Ok(None);
        }

        let kind = SectionId::new(id_byte);
        let /*mut*/ content_length = self.parser.leb128_u32().context("section content size")?;

        let id = if let Some(id_number) = kind {
            SectionKind::Id(id_number)
        } else {
            todo!("custom sections not yet supported")
            // let name_start = self.parser.position()?;
            // let name = todo!();
            // let name_end = self.parser.position()?;
            // content_length -= name_end - name_start;
            //SectionKind::Custom(todo!())
        };

        let contents = self.parser.fork()?;

        self.parser
            .skip_exact(u64::from(content_length))
            .context("section content")?;

        Ok(Some(Section {
            kind: id,
            contents,
            length: content_length,
        }))
    }
}

impl<I: Input> Iterator for SectionSequence<I> {
    type Item = Result<Section<I::Fork>>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.parse().transpose()
    }
}
