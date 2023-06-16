use crate::{
    component::IndexVector,
    index::TypeIdx,
    input::Input,
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
pub struct FunctionSection<I: Input> {
    indices: IndexVector<TypeIdx, u64, I>,
}

impl<I: Input> From<IndexVector<TypeIdx, u64, I>> for FunctionSection<I> {
    #[inline]
    fn from(indices: IndexVector<TypeIdx, u64, I>) -> Self {
        Self { indices }
    }
}

impl<I: Input> FunctionSection<I> {
    /// Uses the given [`Input`] to read the contents of the *function section* of a module, which
    /// begins at the given `offset`.
    #[inline]
    pub fn new(offset: u64, input: I) -> Result<Self> {
        IndexVector::parse(offset, input)
            .context("at start of function section")
            .map(Self::from)
    }

    /// Gets the expected remaining number of entries in the *function section* that have yet to be
    /// parsed.
    #[inline]
    pub fn remaining_count(&self) -> u32 {
        self.indices.remaining_count()
    }

    pub(super) fn borrowed(&self) -> FunctionSection<&I> {
        FunctionSection {
            indices: self.indices.borrowed(),
        }
    }
}

impl<I: Input> Iterator for FunctionSection<I> {
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

impl<I: Clone + Input> core::iter::FusedIterator for FunctionSection<I> {}

impl<I: Input> core::fmt::Debug for FunctionSection<I> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(&self.indices, f)
    }
}
