use crate::allocator::{Allocator, OwnOrRef};
use crate::parser::input::Input;
use crate::parser::{Parser, Result, ResultExt};

mod section_kind;

pub use section_kind::{SectionId, SectionKind};

/// Represents a
/// [WebAssembly section](https://webassembly.github.io/spec/core/binary/modules.html#sections).
#[derive(Debug)]
pub struct Section<I: Input, S: AsRef<str>> {
    kind: SectionKind<S>,
    length: u64,
    contents: Parser<I>,
}

impl<I: Input, S: AsRef<str>> Section<I, S> {
    /// Gets the
    /// [*id* or custom section name](https://webassembly.github.io/spec/core/binary/modules.html#sections)
    /// for this section.
    #[inline]
    pub fn kind(&self) -> &SectionKind<S> {
        &self.kind
    }

    /// Gets the length, in bytes, of the content of the section.
    ///
    /// Note that for custom sections, this does **not** include the section name.
    #[inline]
    pub fn length(&self) -> u64 {
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
pub struct SectionSequence<I: Input, A: Allocator> {
    parser: Parser<I>,
    allocator: A,
    buffer: A::Buf,
}

impl<I: Input, A: Allocator> SectionSequence<I, A> {
    /// Uses the given [`Parser<I>`] to read a sequence of sections with the given [`Allocator`].
    pub fn new_with_allocator(parser: Parser<I>, allocator: A) -> Self {
        Self {
            parser,
            buffer: allocator.allocate_buffer(),
            allocator,
        }
    }

    /// Uses the given [`Input`] to read a sequence of sections with the given [`Allocator`].
    pub fn from_input_with_allocator(input: I, allocator: A) -> Self {
        Self::new_with_allocator(Parser::new(input), allocator)
    }

    fn parse(&mut self) -> Result<Option<Section<I::Fork, A::String>>> {
        let mut id_byte = 0u8;

        let id_length = self
            .parser
            .bytes(core::slice::from_mut(&mut id_byte))
            .context("section id byte")?;

        if id_length == 0 {
            return Ok(None);
        }

        let kind = SectionId::new(id_byte);
        let mut content_length =
            u64::from(self.parser.leb128_u32().context("section content size")?);

        let id = if let Some(id_number) = kind {
            SectionKind::Id(id_number)
        } else {
            let name_start = self.parser.position()?;
            let name = self
                .parser
                .name(&mut self.buffer)
                .context("custom section name")?;
            let name_end = self.parser.position()?;
            content_length -= name_end - name_start;
            SectionKind::Custom(
                if let Some(known) = section_kind::cached_custom_name(name) {
                    OwnOrRef::Reference(known)
                } else {
                    OwnOrRef::Owned(self.allocator.allocate_string(name))
                },
            )
        };

        let contents = self.parser.fork()?;

        self.parser
            .skip_exact(content_length)
            .context("section content")?;

        Ok(Some(Section {
            kind: id,
            contents,
            length: content_length,
        }))
    }
}

#[cfg(feature = "alloc")]
impl<I: Input> SectionSequence<I, crate::allocator::Global> {
    /// Uses the given [`Parser<I>`] to read a sequence of sections.
    pub fn new(parser: Parser<I>) -> Self {
        Self::new_with_allocator(parser, Default::default())
    }

    /// Uses the given [`Input`] to read a sequence of sections.
    pub fn from_input(input: I) -> Self {
        Self::new(Parser::new(input))
    }
}

impl<I: Input, A: Allocator> Iterator for SectionSequence<I, A> {
    type Item = Result<Section<I::Fork, A::String>>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.parse().transpose()
    }
}

impl<I, A> core::fmt::Debug for SectionSequence<I, A>
where
    I: Input + core::fmt::Debug,
    A: Allocator + core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SectionSequence")
            .field("parser", &self.parser)
            .field("allocator", &self.allocator)
            .finish()
    }
}
