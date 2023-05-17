use crate::allocator::{Allocator, Buffer, OwnOrRef};
use crate::bytes::{Bytes, Window};
use crate::parser::{self, Result, ResultExt};
use core::fmt::Debug;

mod section_kind;

pub use section_kind::{SectionId, SectionKind};

pub(crate) use section_kind::section_id as id;

/// Represents a
/// [WebAssembly section](https://webassembly.github.io/spec/core/binary/modules.html#sections).
#[derive(Clone)]
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
            kind: self.kind.borrowed(),
            contents: self.contents.borrowed(),
        }
    }
}

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

/// Represents the
/// [sequence of sections](https://webassembly.github.io/spec/core/binary/modules.html#binary-module)
/// in a WebAssembly module.
#[must_use]
pub struct SectionSequence<B: Bytes, A: Allocator> {
    offset: u64,
    bytes: B,
    allocator: A,
    buffer: A::Buf,
}

impl<B: Bytes, A: Allocator> SectionSequence<B, A> {
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

    /// Parses the next section. If there are no more sections remaining, returns `Ok(None)`.
    ///
    /// # Errors
    ///
    /// Returns an error if the [`Bytes`] could not be read, or if a structure was not formatted
    /// correctly.
    pub fn parse(&mut self) -> Result<Option<Section<&B, A::String>>> {
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

            self.buffer.clear();
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

        let contents = Window::new(&self.bytes, self.offset, content_length);

        // TODO: Duplicate code w/ leb128, increment offset
        // self.parser
        //     .skip_exact(content_length)
        //     .context("section content")?;
        self.offset += content_length;

        Ok(Some(Section { kind: id, contents }))
    }
}

#[cfg(feature = "alloc")]
impl<B: Bytes> SectionSequence<B, crate::allocator::Global> {
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
        match self.parse() {
            Ok(None) => None,
            Ok(Some(section)) => Some(Ok(Section {
                kind: section.kind,
                contents: section.contents.cloned(),
            })),
            Err(e) => Some(Err(e)),
        }
    }
}

impl<B: Bytes + Debug, A: Allocator> Debug for SectionSequence<B, A> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let clone = SectionSequence {
            offset: self.offset,
            bytes: &self.bytes,
            buffer: self.allocator.allocate_buffer(),
            allocator: &self.allocator,
        };

        f.debug_list().entries(clone).finish()
    }
}
