use crate::buffer::Buffer;
use crate::bytes::{Bytes, Window};
use crate::parser::{self, Result, ResultExt};
use core::fmt::Debug;

mod section_kind;

pub use section_kind::{SectionId, SectionKind};

pub(crate) use section_kind::section_id as id;

/// Represents a
/// [WebAssembly section](https://webassembly.github.io/spec/core/binary/modules.html#sections).
#[derive(Clone, Copy)]
pub struct Section<B: Bytes, S: AsRef<str>> {
    kind: SectionKind<S>,
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
        self.contents.length()
    }

    /// Returns a [`Window`] into the contents of the section.
    #[inline]
    pub fn contents(&self) -> Window<&B> {
        self.contents.borrowed()
    }

    /// Consumes the section, returning its contents as a [`Window`].
    ///
    /// The offset to the first byte of the section's content can be obtained by calling
    /// [`Window::base`].
    #[inline]
    pub fn into_contents(self) -> Window<B> {
        self.contents
    }

    #[cfg(feature = "alloc")]
    fn borrowed(&self) -> Section<&B, &str> {
        Section {
            kind: self.kind.into_borrowed(),
            contents: self.contents.borrowed(),
        }
    }
}

/*
impl<B: Bytes + Debug, S: AsRef<str>> Debug for Section<B, S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        #[cfg(feature = "alloc")]
        if let Ok(Ok(known)) = crate::component::KnownSection::try_from_with_allocator(
            self.borrowed(),
            crate::allocator::Global,
        ) {
            return Debug::fmt(&known, f);
        }

        f.debug_struct("Section")
            .field("kind", &self.kind)
            .field("contents", &self.contents)
            .finish()
    }
}
*/

/// Represents the
/// [sequence of sections](https://webassembly.github.io/spec/core/binary/modules.html#binary-module)
/// in a WebAssembly module.
#[derive(Clone, Copy)]
#[must_use]
pub struct SectionSequence<B: Bytes> {
    offset: u64,
    bytes: B,
}

impl<B: Bytes> SectionSequence<B> {
    /// Uses the given [`Bytes`] to read a sequence of sections starting at the specified `offset`.
    pub fn new(offset: u64, bytes: B) -> Self {
        Self { offset, bytes }
    }

    /// Parses the next section. If there are no more sections remaining, returns `Ok(None)`.
    ///
    /// # Errors
    ///
    /// Returns an error if the [`Bytes`] could not be read, or if a structure was not formatted
    /// correctly.
    pub fn parse_with_buffer<'n, N: Buffer>(
        &mut self,
        name_buffer: &'n mut N,
    ) -> Result<Option<Section<&B, &'n str>>> {
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

            name_buffer.clear();
            let name: &str = parser::name(&mut self.offset, &self.bytes, name_buffer)
                .context("custom section name")?;

            content_length -= self.offset - name_start;

            SectionKind::Custom(name)
        };

        let contents = Window::new(&self.bytes, self.offset, content_length);

        // TODO: Duplicate code w/ leb128, increment offset
        // self.parser
        //     .skip_exact(content_length)
        //     .context("section content")?;
        self.offset += content_length;

        Ok(Some(Section { kind: id, contents }))
    }
}

/*
impl<B: Bytes> core::fmt::Debug for SectionSequence
*/
