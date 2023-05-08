use crate::component::MemType;
use crate::parser::{input::Input, Decoder, Result, ResultExt, SimpleParse, Vector};

/// Represents the
/// [**mems** component](https://webassembly.github.io/spec/core/syntax/modules.html#memories) of a
/// WebAssembly module, stored in and parsed from the
/// [*memory section*](https://webassembly.github.io/spec/core/binary/modules.html#memory-section).
pub struct MemsComponent<I: Input> {
    limits: Vector<I, SimpleParse<MemType>>,
}

impl<I: Input> From<Vector<I, SimpleParse<MemType>>> for MemsComponent<I> {
    #[inline]
    fn from(limits: Vector<I, SimpleParse<MemType>>) -> Self {
        Self { limits }
    }
}

impl<I: Input> MemsComponent<I> {
    /// Uses the given [`Decoder<I>`] to read the contents of the *memory section* of a module.
    pub fn new(input: Decoder<I>) -> Result<Self> {
        Vector::new(input, Default::default())
            .map(Self::from)
            .context("memory section")
    }
}

impl<I: Input> core::iter::Iterator for MemsComponent<I> {
    type Item = Result<MemType>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.limits.next().map(|r| r.context("memory section"))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.limits.size_hint()
    }
}

impl<I: Input> core::fmt::Debug for MemsComponent<I> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        crate::component::debug_section_contents(self.limits.try_clone(), f)
    }
}
