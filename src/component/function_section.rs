use crate::bytes::Bytes;
use crate::index::TypeIdx;
use crate::parser::{Result, ResultExt, SimpleParse, Vector};

/// Represents the
/// [*function section*](https://webassembly.github.io/spec/core/binary/modules.html#function-section),
/// which corresponds to the
/// [**type**](https://webassembly.github.io/spec/core/syntax/modules.html#syntax-func) of each
/// function in the
/// [**funcs** component](https://webassembly.github.io/spec/core/syntax/modules.html#syntax-func)
/// of a WebAssembly module.
#[derive(Clone, Copy)]
pub struct FunctionSection<B: Bytes> {
    indices: Vector<u64, B, SimpleParse<TypeIdx>>,
}

impl<B: Bytes> From<Vector<u64, B, SimpleParse<TypeIdx>>> for FunctionSection<B> {
    #[inline]
    fn from(indices: Vector<u64, B, SimpleParse<TypeIdx>>) -> Self {
        Self { indices }
    }
}

impl<B: Bytes> FunctionSection<B> {
    /// Uses the given [`Bytes`] to read the contents of the *function section* of a module, which
    /// begins at the given `offset`.
    #[inline]
    pub fn new(offset: u64, bytes: B) -> Result<Self> {
        Vector::new(offset, bytes, Default::default()).map(Self::from)
    }

    /// Gets the expected remaining number of entries in the *function section* that have yet to be
    /// parsed.
    #[inline]
    pub fn len(&self) -> u32 {
        self.indices.len()
    }

    /// Returns a value indicating if the *function section* is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    pub(super) fn borrowed(&self) -> FunctionSection<&B> {
        FunctionSection {
            indices: self.indices.by_reference(),
        }
    }
}

impl<B: Bytes> Iterator for FunctionSection<B> {
    type Item = Result<TypeIdx>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.indices.next().map(|r| r.context("function section"))
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
