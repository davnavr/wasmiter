use crate::{
    bytes::Bytes,
    index::Index,
    parser::{self, name::Name},
};

/// A [**nameassoc**](https://webassembly.github.io/spec/core/appendix/custom.html#name-maps)
/// associates an [`Index`] with a [`Name`].
#[derive(Clone, Copy)]
pub struct NameAssoc<I: Index, B: Bytes> {
    index: I,
    name: Name<B>,
}

impl<I: Index, B: Bytes> NameAssoc<I, B> {
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

impl<I: Index, B: Bytes> core::fmt::Debug for NameAssoc<I, B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("NameAssoc")
            .field("index", &self.index)
            .field("name", &self.name)
            .finish()
    }
}
