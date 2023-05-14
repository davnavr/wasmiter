use crate::allocator::{Allocator, OwnOrRef};
use crate::bytes::{Bytes, Window};
use crate::parser::{self, Offset, Result, ResultExt};

mod section_kind;

pub use section_kind::{SectionId, SectionKind};

pub(crate) use section_kind::section_id as id;

/// Represents a
/// [WebAssembly section](https://webassembly.github.io/spec/core/binary/modules.html#sections).
#[derive(Debug)]
pub struct Section<B: Bytes, S: AsRef<str>> {
    kind: SectionKind<S>,
    length: u64,
    contents: Window<B>,
}

impl<B: Bytes, S: AsRef<str>> Section<B, S> {
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

    /// Consumes the section, returning its contents as a [`Window`].
    ///
    /// The offset to the first byte of the section's content can be obtained by calling
    /// [`Window::base`].
    #[inline]
    pub fn into_contents(self) -> Window<B> {
        self.contents
    }
}

/// Represents the
/// [sequence of sections](https://webassembly.github.io/spec/core/binary/modules.html#binary-module)
/// in a WebAssembly module.
#[must_use]
pub struct SectionSequence<B: Bytes + Clone, A: Allocator> {
    offset: u64,
    bytes: B,
    allocator: A,
    buffer: A::Buf,
}

impl<B: Bytes + Clone, A: Allocator> SectionSequence<B, A> {
    /// Uses the given [`Bytes`] to read a sequence of sections with the given [`Allocator`]
    /// starting at the specified `offset`.
    pub fn new_with_allocator(offset: u64, bytes: B, allocator: A) -> Self {
        Self {
            offset,
            bytes,
            buffer: allocator.allocate_buffer(),
            allocator,
        }
    }

    fn parse(&mut self) -> Result<Option<Section<B, A::String>>> {
        let id_byte = if let Some(id) = parser::one_byte(&mut self.offset, &self.bytes)? {
            id
        } else {
            return Ok(None);
        };

        let kind = SectionId::new(id_byte);
        let mut content_length = u64::from(
            parser::leb128::u32(&mut self.offset, &self.bytes).context("section content size")?,
        );

        let id = if let Some(id_number) = kind {
            SectionKind::Id(id_number)
        } else {
            let name_start = self.offset;
            let name = parser::name(&mut self.offset, &self.bytes, &mut self.buffer)
                .context("custom section name")?;

            content_length -= self.offset - name_start;

            SectionKind::Custom(
                if let Some(known) = section_kind::cached_custom_name(name) {
                    OwnOrRef::Reference(known)
                } else {
                    OwnOrRef::Owned(self.allocator.allocate_string(name))
                },
            )
        };

        let contents = Window::new(self.bytes.clone(), self.offset, content_length);

        // TODO: Duplicate code w/ leb128, increment offset
        // self.parser
        //     .skip_exact(content_length)
        //     .context("section content")?;
        self.offset += content_length;

        Ok(Some(Section {
            kind: id,
            contents,
            length: content_length,
        }))
    }
}

#[cfg(feature = "alloc")]
impl<B: Bytes + Clone> SectionSequence<B, crate::allocator::Global> {
    /// Uses the given [`Bytes`] to read a sequence of sections starting at the specified `offset`.
    #[inline]
    pub fn new(offset: u64, bytes: B) -> Self {
        Self::new_with_allocator(offset, bytes, Default::default())
    }
}

impl<B: Bytes + Clone, A: Allocator> Iterator for SectionSequence<B, A> {
    type Item = Result<Section<B, A::String>>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.parse().transpose()
    }
}

impl<B, A> core::fmt::Debug for SectionSequence<B, A>
where
    B: Bytes + Clone,
    A: Allocator,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SectionSequence")
            .field("offset", &self.offset)
            .finish()
    }
}
