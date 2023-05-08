use crate::component::FuncIdx;
use crate::parser::{input::Input, Decoder, Result, SimpleParse, Vector};

/// Represents the
/// [*function section*](https://webassembly.github.io/spec/core/binary/modules.html#function-section),
/// which corresponds to the
/// [**type**](https://webassembly.github.io/spec/core/syntax/modules.html#syntax-func) of each
/// function in the
/// [**funcs** component](https://webassembly.github.io/spec/core/syntax/modules.html#syntax-func)
/// of a WebAssembly module.
pub struct FunctionSection<I: Input> {
    indices: Vector<I, SimpleParse<FuncIdx>>,
}

impl<I: Input> From<Vector<I, SimpleParse<FuncIdx>>> for FunctionSection<I> {
    #[inline]
    fn from(indices: Vector<I, SimpleParse<FuncIdx>>) -> Self {
        Self { indices }
    }
}

impl<I: Input> FunctionSection<I> {
    /// Uses the given [`Decoder<I>`] to read the contents of the *function section* of a module.
    #[inline]
    pub fn new(input: Decoder<I>) -> Result<Self> {
        Vector::new(input, Default::default()).map(Self::from)
    }
}

impl<I: Input> Iterator for FunctionSection<I> {
    type Item = Result<FuncIdx>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.indices.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.indices.size_hint()
    }
}

impl<I: Input> core::fmt::Debug for FunctionSection<I> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        crate::component::debug_section_contents(self.indices.try_clone(), f)
    }
}
