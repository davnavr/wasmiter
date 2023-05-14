use crate::bytes::Bytes;
use crate::component::MemType;
use crate::parser::{Result, ResultExt, SimpleParse, Vector};

/// Represents the
/// [**mems** component](https://webassembly.github.io/spec/core/syntax/modules.html#memories) of a
/// WebAssembly module, stored in and parsed from the
/// [*memory section*](https://webassembly.github.io/spec/core/binary/modules.html#memory-section).
pub struct MemsComponent<B: Bytes> {
    limits: Vector<u64, B, SimpleParse<MemType>>,
}

impl<B: Bytes> From<Vector<u64, B, SimpleParse<MemType>>> for MemsComponent<B> {
    #[inline]
    fn from(limits: Vector<u64, B, SimpleParse<MemType>>) -> Self {
        Self { limits }
    }
}

impl<B: Bytes> MemsComponent<B> {
    /// Uses the given [`Bytes`] to read the contents of the *memory section* of a module, starting
    /// at the specified `offset`.
    pub fn new(offset: u64, bytes: B) -> Result<Self> {
        Vector::new(offset, bytes, Default::default())
            .map(Self::from)
            .context("memory section")
    }
}

impl<B: Bytes> core::iter::Iterator for MemsComponent<B> {
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

impl<B: Bytes> core::fmt::Debug for MemsComponent<B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(&self.limits, f)
    }
}
