use crate::{
    bytes::Bytes,
    parser::{self, ResultExt as _},
};

/// Represents a
/// [**tag**](https://webassembly.github.io/exception-handling/core/syntax/modules.html#tags) in
/// the
/// [*tag section*](https://webassembly.github.io/exception-handling/core/binary/modules.html#tag-section)
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Tag {
    /// Describes an exception that can be thrown or caught, introduced as part of the
    /// [exception handling proposal](https://github.com/WebAssembly/exception-handling/).
    Exception(crate::index::TypeIdx),
}

#[derive(Clone, Copy)]
struct ParseTag;

impl parser::Parse for ParseTag {
    type Output = Tag;

    fn parse<B: Bytes>(&mut self, offset: &mut u64, bytes: B) -> parser::Result<Self::Output> {
        let attribute = parser::one_byte_exact(offset, &bytes)?;
        if attribute != 0 {
            crate::parser_bad_format!("{attribute:#04X} is not a valid tag attribute");
        }
        crate::component::index(offset, bytes).map(Tag::Exception)
    }
}

/// Represents the
/// [**tags** component](https://webassembly.github.io/exception-handling/core/syntax/modules.html#tags) of a
/// WebAssembly module, stored in and parsed from the
/// [*tag section*](https://webassembly.github.io/exception-handling/core/binary/modules.html#tag-section).
#[derive(Clone, Copy)]
pub struct TagsComponent<B: Bytes> {
    tags: parser::Vector<u64, B, ParseTag>,
}

impl<B: Bytes> TagsComponent<B> {
    /// Uses the given [`Bytes`] to read the contents of the *type section* of a module, starting
    /// at the specified `offset`.
    pub fn new(offset: u64, bytes: B) -> parser::Result<Self> {
        Ok(Self {
            tags: parser::Vector::new(offset, bytes, ParseTag).context("tag section")?,
        })
    }

    /// Gets the expected remaining number of tags that have yet to be parsed.
    #[inline]
    pub fn len(&self) -> u32 {
        self.tags.len()
    }

    /// Returns a value indicating if the *tag section* is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.tags.is_empty()
    }

    /// Parses the next [`Tag`] in the section.
    pub fn parse<T, E, F>(&mut self, f: F) -> Result<Option<T>, E>
    where
        E: From<parser::Error>,
        F: FnOnce(&mut Tag) -> Result<T, E>,
    {
        match self.tags.next() {
            None => Ok(None),
            Some(Ok(mut tag)) => f(&mut tag).map(Some),
            Some(Err(e)) => Err(e.into()),
        }
    }

    pub(crate) fn borrowed(&self) -> TagsComponent<&B> {
        TagsComponent {
            tags: self.tags.by_reference(),
        }
    }
}

impl<B: Bytes> core::fmt::Debug for TagsComponent<B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(&self.tags, f)
    }
}
