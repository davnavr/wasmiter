use crate::{
    bytes::Bytes,
    index::Index,
    parser::{self, name::Name, ResultExt as _},
};

/// A [**nameassoc**](https://webassembly.github.io/spec/core/appendix/custom.html#name-maps)
/// associates an [`Index`] with a [`Name`].
#[derive(Clone, Copy)]
pub struct NameAssoc<I: Index, B: Bytes> {
    index: I,
    name: Name<B>,
}

impl<I: Index, B: Bytes> NameAssoc<I, B> {
    /// Parses a [`NameAssoc`].
    pub fn parse(offset: &mut u64, bytes: B) -> parser::Result<Self> {
        Ok(Self {
            index: crate::component::index(offset, &bytes).context("index of nameassoc pair")?,
            name: parser::name::parse(offset, bytes).context("name of nameassoc pair")?,
        })
    }

    /// Gets the index.
    #[inline]
    pub fn index(&self) -> I {
        self.index
    }

    /// Gets the name.
    #[inline]
    pub fn name(&self) -> &Name<B> {
        &self.name
    }
}

impl<I: Index, B: Clone + Bytes> NameAssoc<I, &B> {
    pub(super) fn dereferenced(&self) -> NameAssoc<I, B> {
        NameAssoc {
            index: self.index,
            name: self.name.really_cloned(),
        }
    }
}

impl<I: Index, B: Bytes> core::fmt::Debug for NameAssoc<I, B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("NameAssoc")
            .field("index", &self.index)
            .field("name", &self.name)
            .finish()
    }
}
