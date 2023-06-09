//! Contains types for reading WebAssembly sections.
//!
//! A sequence of sections is a common structure in the WebAssembly binary format, used not only in
//! the
//! [encoding for modules](https://webassembly.github.io/spec/core/binary/modules.html#binary-section),
//! but also in some custom sections. Examples include the
//! [`name` custom section](crate::custom::name), and in the
//! [`dylink.0` custom section described in the Dynamic Linking document](https://github.com/WebAssembly/tool-conventions/blob/main/DynamicLinking.md).

use crate::bytes::{Bytes, Window};
use crate::parser::{self, Result, ResultExt};
use core::fmt::Debug;

mod debug_module;
mod display_module;

pub mod id;

pub use debug_module::{DebugModule, DebugModuleSection};
pub use display_module::DisplayModule;

/// Represents a
/// [WebAssembly section](https://webassembly.github.io/spec/core/binary/modules.html#sections),
/// typically a
/// [section within a module](https://webassembly.github.io/spec/core/binary/modules.html#binary-section).
///
/// To interpret the contents of a WebAssembly module section, consider using
/// [`component::KnownSection::interpret`](crate::component::KnownSection::interpret), or in the
/// case of a custom section, [`custom::CustomSection::interpret`](crate::custom::CustomSection::interpret).
#[derive(Clone, Copy)]
pub struct Section<B: Bytes> {
    id: u8,
    contents: Window<B>,
}

impl<B: Bytes> Section<B> {
    /// Creates a new [`Section`] with the given
    /// [*id*](https://webassembly.github.io/spec/core/binary/modules.html#sections) and binary
    /// `contents`.
    pub fn new(id: u8, contents: Window<B>) -> Self {
        Self { id, contents }
    }

    /// Gets the [*id*](https://webassembly.github.io/spec/core/binary/modules.html#sections) for
    /// this section.
    #[inline]
    pub fn id(&self) -> u8 {
        self.id
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
            id: self.id,
            contents: self.contents.borrowed(),
        }
    }

    /// Returns a [`Debug`] implementation that attempts to interpret the contents as a WebAssembly
    /// module section.
    #[inline]
    pub fn debug_module(&self) -> DebugModuleSection<'_, B> {
        DebugModuleSection::new(self)
    }
}

impl<B: Bytes + Clone> Section<&B> {
    /// Returns a version of the [`Section`] with the contents cloned.
    pub fn cloned(&self) -> Section<B> {
        Section {
            id: self.id,
            contents: self.contents.cloned(),
        }
    }
}

impl<B: Bytes> Debug for Section<B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Section")
            .field("id", &self.id)
            .field("contents", &self.contents)
            .finish()
    }
}

/// Represents a sequence of WebAssembly [`Section`]s, commonly the
/// [sequence of sections in a WebAssembly module](https://webassembly.github.io/spec/core/binary/modules.html#binary-module).
#[derive(Clone, Copy)]
#[must_use]
pub struct SectionSequence<B: Bytes> {
    offset: u64,
    bytes: B,
}

impl<B: Bytes> SectionSequence<B> {
    /// Uses the given [`Bytes`] to parse a sequence of sections starting at the specified `offset`.
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
        let id = if let Some(value) = parser::one_byte(&mut self.offset, &self.bytes)? {
            value
        } else {
            return Ok(None);
        };

        let content_length = u64::from(
            parser::leb128::u32(&mut self.offset, &self.bytes).context("section content size")?,
        );

        let contents = Window::new(&self.bytes, self.offset, content_length);

        // TODO: Duplicate code w/ leb128, increment offset
        // self.parser
        //     .skip_exact(content_length)
        //     .context("section content")?;
        self.offset += content_length;

        Ok(Some(Section { id, contents }))
    }

    pub(crate) fn borrowed(&self) -> SectionSequence<&B> {
        SectionSequence {
            offset: self.offset,
            bytes: &self.bytes,
        }
    }

    /// Returns a [`Debug`] implementation that attempts to interpret the sequence of sections as a
    /// WebAssembly module's sections.
    #[inline]
    pub fn debug_module(&self) -> DebugModule<'_, B> {
        DebugModule::new(self)
    }

    /// Returns a [`Display`](core::fmt::Display) implementation that attempts to interpret the
    /// sequence of sections as a WebAssembly module's sections, and writing the corresponding
    /// [WebAssembly text](https://webassembly.github.io/spec/core/text/index.html).
    #[inline]
    pub fn display_module(&self) -> DisplayModule<'_, B> {
        DisplayModule::new(self)
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
