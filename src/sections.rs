//! Contains types for reading the sections of a WebAssembly module.

use crate::buffer::{Arena, Buffer};
use crate::bytes::{Bytes, Window};
use crate::parser::{self, Result, ResultExt};
use core::fmt::Debug;

#[cfg(feature = "alloc")]
use crate::buffer::GlobalArena;

mod section_kind;

pub use section_kind::{section_id as id, SectionId, SectionKind};

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

    /// Returns a borrowed version of the [`Section`].
    pub fn borrowed(&self) -> Section<&B, &str> {
        Section {
            kind: self.kind.into_borrowed(),
            contents: self.contents.borrowed(),
        }
    }
}

impl<B: Bytes + Clone, S: AsRef<str> + Clone> Section<&B, S> {
    /// Returns a version of the [`Section`] with the contents cloned.
    pub fn cloned(&self) -> Section<B, S> {
        Section {
            kind: self.kind.clone(),
            contents: self.contents.cloned(),
        }
    }
}

impl<B: Bytes, S: AsRef<str>> Debug for Section<B, S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        #[cfg(feature = "alloc")]
        if let Ok(Ok(known)) = crate::component::KnownSection::try_from_section(self.borrowed()) {
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

    fn borrowed(&self) -> SectionSequence<&B> {
        SectionSequence {
            offset: self.offset,
            bytes: &self.bytes,
        }
    }

    /// Returns an iterator over the sections, using the provided [`Arena`] when allocating custom
    /// section names.
    pub fn iter_with_arena<A: Arena>(&self, arena: A) -> SectionsIter<&B, A> {
        SectionsIter::with_arena(self.borrowed(), arena)
    }

    /// Returns an iterator over the sections.
    #[cfg(feature = "alloc")]
    pub fn iter(&self) -> SectionsIter<&B, GlobalArena> {
        self.iter_with_arena(GlobalArena)
    }
}

/// Provides an [`Iterator`] implementation for a [`SectionSequence`].
#[derive(Clone, Copy)]
pub struct SectionsIter<B: Bytes, A: Arena> {
    sections: SectionSequence<B>,
    buffer: A::Buf,
    arena: A,
}

impl<B: Bytes, A: Arena> SectionsIter<B, A> {
    /// Creates an [`Iterator`] over the [`SectionSequence`], using the given [`Arena`] when
    /// allocating custom section names.
    pub fn with_arena(sections: SectionSequence<B>, arena: A) -> Self {
        Self {
            sections,
            buffer: arena.allocate_buffer(0),
            arena,
        }
    }
}

#[cfg(feature = "alloc")]
impl<B: Bytes> SectionsIter<B, GlobalArena> {
    /// Creates an [`Iterator`] over the [`SectionSequence`].
    pub fn new(sections: SectionSequence<B>) -> Self {
        Self::with_arena(sections, GlobalArena)
    }
}

impl<B: Bytes + Clone, A: Arena> Iterator for SectionsIter<B, A> {
    type Item = Result<Section<B, A::String>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.sections.parse_with_buffer(&mut self.buffer) {
            Ok(Some(section)) => Some(Ok(Section {
                kind: match section.kind {
                    SectionKind::Id(id) => SectionKind::Id(id),
                    SectionKind::Custom(name) => {
                        SectionKind::Custom(self.arena.allocate_string(name))
                    }
                },
                contents: section.contents.cloned(),
            })),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

impl<B: Bytes, A: Arena> Debug for SectionsIter<B, A> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let sections = SectionsIter {
            sections: self.sections.borrowed(),
            buffer: self
                .arena
                .allocate_buffer(self.buffer.capacity().unwrap_or_default()),
            arena: &self.arena,
        };

        f.debug_list().entries(sections).finish()
    }
}

#[cfg(feature = "alloc")]
impl<B: Bytes + Clone> IntoIterator for SectionSequence<B> {
    type IntoIter = SectionsIter<B, GlobalArena>;

    type Item = Result<Section<B, alloc::boxed::Box<str>>>;

    fn into_iter(self) -> Self::IntoIter {
        SectionsIter::new(self)
    }
}

impl<B: Bytes> Debug for SectionSequence<B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        #[cfg(feature = "alloc")]
        return {
            let mut buffer = smallvec::smallvec_inline![0u8; 64];
            let mut list = f.debug_list();

            let mut sequence = self.borrowed();
            while let Some(section) = sequence.parse_with_buffer(&mut buffer).transpose() {
                list.entry(&section);
            }

            list.finish()
        };

        #[cfg(not(feature = "alloc"))]
        return f
            .debug_struct("SectionSequence")
            .field("offset", &self.offset)
            .field("bytes", &crate::bytes::DebugBytes::from(&self.bytes))
            .finish();
    }
}
