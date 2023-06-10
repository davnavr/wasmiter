use crate::{
    bytes::Bytes,
    parser::{Result, ResultExt, Vector},
    types::MemType,
};

/// Represents the
/// [**mems** component](https://webassembly.github.io/spec/core/syntax/modules.html#memories) of a
/// WebAssembly module, stored in and parsed from the
/// [*memory section*](https://webassembly.github.io/spec/core/binary/modules.html#memory-section).
///
/// Note that defining more than one memory requires the
/// [multi-memory proposal](https://github.com/WebAssembly/multi-memory).
#[derive(Clone, Copy)]
pub struct MemsComponent<B: Bytes> {
    types: Vector<u64, B>,
}

impl<B: Bytes> From<Vector<u64, B>> for MemsComponent<B> {
    #[inline]
    fn from(types: Vector<u64, B>) -> Self {
        Self { types }
    }
}

impl<B: Bytes> MemsComponent<B> {
    /// Uses the given [`Bytes`] to read the contents of the *memory section* of a module, starting
    /// at the specified `offset`.
    pub fn new(offset: u64, bytes: B) -> Result<Self> {
        Vector::parse(offset, bytes)
            .context("at start of memory section")
            .map(Self::from)
    }

    /// Gets the expected remaining number of entries in the *memory section* that have yet to be
    /// parsed.
    #[inline]
    pub fn remaining_count(&self) -> u32 {
        self.types.remaining_count()
    }

    pub(crate) fn borrowed(&self) -> MemsComponent<&B> {
        MemsComponent {
            types: self.types.borrowed(),
        }
    }
}

impl<B: Bytes> core::iter::Iterator for MemsComponent<B> {
    type Item = Result<MemType>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.types.advance(|offset, bytes| {
            crate::component::mem_type(offset, bytes).context("within memory section")
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.types.size_hint()
    }
}

impl<B: Clone + Bytes> core::iter::FusedIterator for MemsComponent<B> {}

impl<B: Bytes> core::fmt::Debug for MemsComponent<B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(&self.types, f)
    }
}
