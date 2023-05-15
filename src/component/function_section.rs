use crate::bytes::Bytes;
use crate::component::FuncIdx;
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
    indices: Vector<u64, B, SimpleParse<FuncIdx>>,
}

impl<B: Bytes> From<Vector<u64, B, SimpleParse<FuncIdx>>> for FunctionSection<B> {
    #[inline]
    fn from(indices: Vector<u64, B, SimpleParse<FuncIdx>>) -> Self {
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
}

impl<B: Bytes> Iterator for FunctionSection<B> {
    type Item = Result<FuncIdx>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.indices.next().map(|r| r.context("function section"))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.indices.size_hint()
    }
}

impl<B: Bytes> core::fmt::Debug for FunctionSection<B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(&self.indices, f)
    }
}
