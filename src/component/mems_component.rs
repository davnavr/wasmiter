use crate::{
    bytes::Bytes,
    parser::{Result, ResultExt, SimpleParse, Vector},
    types::MemType,
};

/// Represents the
/// [**mems** component](https://webassembly.github.io/spec/core/syntax/modules.html#memories) of a
/// WebAssembly module, stored in and parsed from the
/// [*memory section*](https://webassembly.github.io/spec/core/binary/modules.html#memory-section).
#[derive(Clone, Copy)]
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

    /// Gets the expected remaining number of entries in the *memory section* that have yet to be
    /// parsed.
    #[inline]
    pub fn len(&self) -> u32 {
        self.limits.len()
    }

    /// Returns a value indicating if the *memory section* is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.limits.is_empty()
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
