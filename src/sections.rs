//! Contains types for reading WebAssembly sections.
//!
//! A sequence of sections is a common structure in the WebAssembly binary format, used not only in
//! the
//! [encoding for modules](https://webassembly.github.io/spec/core/binary/modules.html#binary-section),
//! but also in some custom sections. Examples include the
//! [`name` custom section](crate::custom::name), and in the
//! [`dylink.0` custom section described in the Dynamic Linking document](https://github.com/WebAssembly/tool-conventions/blob/main/DynamicLinking.md).

use crate::input::{BorrowInput, CloneInput, HasInput, Input, Window};
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
/// case of a custom section,
/// [`custom::KnownCustomSection::interpret`](crate::custom::KnownCustomSection::interpret).
#[derive(Clone, Copy)]
pub struct Section<I: Input> {
    id: u8,
    contents: Window<I>,
}

impl<I: Input> Section<I> {
    /// Creates a new [`Section`] with the given
    /// [*id*](https://webassembly.github.io/spec/core/binary/modules.html#sections) and binary
    /// `contents`.
    pub fn new(id: u8, contents: Window<I>) -> Self {
        Self { id, contents }
    }

    /// Gets the [*id*](https://webassembly.github.io/spec/core/binary/modules.html#sections) for
    /// this section.
    #[inline]
    pub fn id(&self) -> u8 {
        self.id
    }

    /// Gets the length, in bytes, of the content of the section.
    #[inline]
    pub fn length(&self) -> u64 {
        self.contents.length()
    }

    /// Returns a [`Window`] into the contents of the section.
    #[inline]
    pub fn contents(&self) -> &Window<I> {
        &self.contents
    }

    /// Consumes the section, returning its contents as a [`Window`].
    ///
    /// The offset to the first byte of the section's content can be obtained by calling
    /// [`Window::base`].
    #[inline]
    pub fn into_contents(self) -> Window<I> {
        self.contents
    }

    /// Returns a [`Debug`] implementation that attempts to interpret the contents as a WebAssembly
    /// module section.
    #[inline]
    pub fn debug_module(&self) -> DebugModuleSection<'_, I> {
        DebugModuleSection::new(self)
    }
}

impl<I: Input> HasInput<I> for Section<I> {
    #[inline]
    fn input(&self) -> &I {
        self.contents.input()
    }
}

impl<I: Input> HasInput<Window<I>> for Section<I> {
    #[inline]
    fn input(&self) -> &Window<I> {
        self.contents()
    }
}

impl<'a, I: Input + 'a> BorrowInput<'a, I> for Section<I> {
    type Borrowed = Section<&'a I>;

    #[inline]
    fn borrow_input(&'a self) -> Self::Borrowed {
        Section {
            id: self.id,
            contents: self.contents.borrow_input(),
        }
    }
}

impl<'a, I: Clone + Input + 'a> CloneInput<'a, I> for Section<&'a I> {
    type Cloned = Section<I>;

    #[inline]
    fn clone_input(&self) -> Section<I> {
        Section {
            id: self.id,
            contents: self.contents.clone_input(),
        }
    }
}

impl<I: Input> Debug for Section<I> {
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
pub struct SectionSequence<I: Input> {
    offset: u64,
    input: I,
}

impl<I: Input> SectionSequence<I> {
    /// Uses the given [`Input`] to parse a sequence of sections starting at the specified `offset`.
    pub fn new(offset: u64, input: I) -> Self {
        Self { offset, input }
    }

    /// Gets the offset of the next section ID byte to be parsed.
    #[inline]
    pub fn offset(&self) -> u64 {
        self.offset
    }

    /// Parses the next section. If there are no more sections remaining, returns `Ok(None)`.
    ///
    /// # Errors
    ///
    /// Returns an error if the [`Bytes`] could not be read, or if a structure was not formatted
    /// correctly.
    pub fn parse(&mut self) -> Result<Option<Section<&I>>> {
        let id = if let Some(value) = parser::one_byte(&mut self.offset, &self.input)? {
            value
        } else {
            return Ok(None);
        };

        let content_length = u64::from(
            parser::leb128::u32(&mut self.offset, &self.input).context("section content size")?,
        );

        let contents = Window::with_offset_and_length(&self.input, self.offset, content_length);

        // TODO: Duplicate code w/ leb128, increment offset
        // self.parser
        //     .skip_exact(content_length)
        //     .context("section content")?;
        self.offset += content_length;

        Ok(Some(Section { id, contents }))
    }

    /// Returns a [`Debug`] implementation that attempts to interpret the sequence of sections as a
    /// WebAssembly module's sections.
    #[inline]
    pub fn debug_module(&self) -> DebugModule<'_, I> {
        DebugModule::new(self)
    }

    /// Returns a [`Display`](core::fmt::Display) implementation that attempts to interpret the
    /// sequence of sections as a WebAssembly module's sections, and writing the corresponding
    /// [WebAssembly text](https://webassembly.github.io/spec/core/text/index.html).
    #[inline]
    pub fn display_module(&self) -> DisplayModule<'_, I> {
        DisplayModule::new(self)
    }
}

impl<I: Input> HasInput<I> for SectionSequence<I> {
    #[inline]
    fn input(&self) -> &I {
        &self.input
    }
}

impl<'a, I: Input + 'a> BorrowInput<'a, I> for SectionSequence<I> {
    type Borrowed = SectionSequence<&'a I>;

    #[inline]
    fn borrow_input(&'a self) -> Self::Borrowed {
        SectionSequence {
            offset: self.offset,
            input: &self.input,
        }
    }
}

impl<I: Clone + Input> Iterator for SectionSequence<I> {
    type Item = Result<Section<I>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.parse() {
            Ok(Some(section)) => Some(Ok(section.clone_input())),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

impl<I: Clone + Input> core::iter::FusedIterator for SectionSequence<I> {}

impl<I: Input> Debug for SectionSequence<I> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.borrow_input()).finish()
    }
}
