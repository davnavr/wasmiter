use crate::{
    bytes::Bytes,
    component::IndexVector,
    index::TypeIdx,
    parser::{Result, ResultExt},
};

/// Represents the
/// [*function section*](https://webassembly.github.io/spec/core/binary/modules.html#function-section),
/// which corresponds to the
/// [**type**](https://webassembly.github.io/spec/core/syntax/modules.html#syntax-func) of each
/// function in the
/// [**funcs** component](https://webassembly.github.io/spec/core/syntax/modules.html#syntax-func)
/// of a WebAssembly module.
#[derive(Clone, Copy)]
pub struct FunctionSection<B: Bytes> {
    indices: IndexVector<TypeIdx, u64, B>,
}

impl<B: Bytes> From<IndexVector<TypeIdx, u64, B>> for FunctionSection<B> {
    #[inline]
    fn from(indices: IndexVector<TypeIdx, u64, B>) -> Self {
        Self { indices }
    }
}

impl<B: Bytes> FunctionSection<B> {
    /// Uses the given [`Bytes`] to read the contents of the *function section* of a module, which
    /// begins at the given `offset`.
    #[inline]
    pub fn new(offset: u64, bytes: B) -> Result<Self> {
        IndexVector::parse(offset, bytes)
            .context("at start of function section")
            .map(Self::from)
    }

    /// Gets the expected remaining number of entries in the *function section* that have yet to be
    /// parsed.
    #[inline]
    pub fn remaining_count(&self) -> u32 {
        self.indices.remaining_count()
    }

    pub(super) fn borrowed(&self) -> FunctionSection<&B> {
        FunctionSection {
            indices: self.indices.borrowed(),
        }
    }
}

impl<B: Bytes> Iterator for FunctionSection<B> {
    type Item = Result<TypeIdx>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.indices
            .next()
            .map(|r| r.context("within function section"))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.indices.size_hint()
    }
}

impl<B: Clone + Bytes> core::iter::FusedIterator for FunctionSection<B> {}

impl<B: Bytes> core::fmt::Debug for FunctionSection<B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(&self.indices, f)
    }
}
