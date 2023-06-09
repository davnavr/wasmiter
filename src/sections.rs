//! Contains types for reading the sections of a WebAssembly module.

use crate::bytes::{Bytes, Window};
use crate::parser::{self, Result, ResultExt};
use core::fmt::Debug;

mod section_kind;

pub use section_kind::{section_id as id, SectionId, SectionKind};

/// Represents a
/// [WebAssembly section](https://webassembly.github.io/spec/core/binary/modules.html#sections).
///
/// To interpret the contents of the section, consider using
/// [`component::KnownSection::interpret`](crate::component::KnownSection::interpret), or in the
/// case of a custom section, [`custom::CustomSection::interpret`](crate::custom::CustomSection::interpret).
#[derive(Clone, Copy)]
pub struct Section<B: Bytes> {
    kind: SectionKind<B>,
    contents: Window<B>,
}

impl<B: Bytes> Section<B> {
    /// Gets the
    /// [*id* or custom section name](https://webassembly.github.io/spec/core/binary/modules.html#sections)
    /// for this section.
    #[inline]
    pub fn kind(&self) -> &SectionKind<B> {
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
    pub fn borrowed(&self) -> Section<&B> {
        Section {
            kind: self.kind.into_borrowed(),
            contents: self.contents.borrowed(),
        }
    }
}

impl<B: Bytes + Clone> Section<&B> {
    /// Returns a version of the [`Section`] with the contents cloned.
    pub fn cloned(&self) -> Section<B> {
        Section {
            kind: self.kind.cloned(),
            contents: self.contents.cloned(),
        }
    }
}

impl<B: Bytes> Debug for Section<B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match crate::component::KnownSection::interpret(self.borrowed()) {
            Ok(known) => Debug::fmt(&known, f),
            Err(custom) => match crate::custom::CustomSection::interpret(custom) {
                Ok(known) => Debug::fmt(&known, f),
                Err(unknown) => f
                    .debug_struct("Section")
                    .field("kind", &unknown.kind)
                    .field("contents", &unknown.contents)
                    .finish(),
            },
        }
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
    pub fn parse(&mut self) -> Result<Option<Section<&B>>> {
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

            let name = parser::name::parse(&mut self.offset, &self.bytes)
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

    pub(crate) fn borrowed(&self) -> SectionSequence<&B> {
        SectionSequence {
            offset: self.offset,
            bytes: &self.bytes,
        }
    }
}

impl<B: Clone + Bytes> Iterator for SectionSequence<B> {
    type Item = Result<Section<B>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.parse() {
            Ok(Some(section)) => Some(Ok(section.cloned())),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

impl<B: Clone + Bytes> core::iter::FusedIterator for SectionSequence<B> {}

impl<B: Bytes> Debug for SectionSequence<B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.borrowed()).finish()
    }
}
