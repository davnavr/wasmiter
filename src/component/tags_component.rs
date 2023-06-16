use crate::{
    input::Input,
    parser::{self, Result, ResultExt as _, Vector},
};

/// Represents a
/// [**tag**](https://webassembly.github.io/exception-handling/core/syntax/modules.html#tags) in
/// the
/// [*tag section*](https://webassembly.github.io/exception-handling/core/binary/modules.html#tag-section).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum Tag {
    /// Describes an exception that can be thrown or caught, introduced as part of the
    /// [exception handling proposal](https://github.com/WebAssembly/exception-handling/).
    Exception(crate::index::TypeIdx),
}

/// Parses a [`Tag`] at the given `offset` into the [`Input`].
pub fn parse<I: Input>(offset: &mut u64, input: &I) -> Result<Tag> {
    let attribute = parser::one_byte_exact(offset, input)?;
    if attribute != 0 {
        crate::parser_bad_format!("{attribute:#04X} is not a valid tag attribute");
    }
    crate::component::index(offset, input).map(Tag::Exception)
}

/// Represents the
/// [**tags** component](https://webassembly.github.io/exception-handling/core/syntax/modules.html#tags) of a
/// WebAssembly module, stored in and parsed from the
/// [*tag section*](https://webassembly.github.io/exception-handling/core/binary/modules.html#tag-section).
#[derive(Clone, Copy)]
pub struct TagsComponent<I: Input> {
    tags: Vector<u64, I>,
}

impl<I: Input> From<Vector<u64, I>> for TagsComponent<I> {
    #[inline]
    fn from(tags: Vector<u64, I>) -> Self {
        Self { tags }
    }
}

impl<I: Input> TagsComponent<I> {
    /// Uses the given [`Input`] to read the contents of the *type section* of a module, starting
    /// at the specified `offset`.
    pub fn new(offset: u64, input: I) -> parser::Result<Self> {
        Vector::parse(offset, input)
            .context("at start of tag section")
            .map(Self::from)
    }

    /// Gets the expected remaining number of tags that have yet to be parsed.
    #[inline]
    pub fn remaining_count(&self) -> u32 {
        self.tags.remaining_count()
    }

    pub(crate) fn borrowed(&self) -> TagsComponent<&I> {
        TagsComponent {
            tags: self.tags.borrowed(),
        }
    }
}

impl<I: Input> Iterator for TagsComponent<I> {
    type Item = Result<Tag>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.tags
            .advance(|offset, bytes| parse(offset, bytes).context("within tag section"))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.tags.size_hint()
    }
}

impl<I: Input> core::iter::FusedIterator for TagsComponent<I> {}

impl<I: Input> core::fmt::Debug for TagsComponent<I> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.borrowed()).finish()
    }
}
