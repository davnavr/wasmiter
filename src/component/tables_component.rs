use crate::{
    bytes::Bytes,
    parser::{Result, ResultExt, Vector},
    types::TableType,
};

/// Represents the
/// [**tables** component](https://webassembly.github.io/spec/core/syntax/modules.html#tables) of a
/// WebAssembly module, stored in and parsed from the
/// [*tables section*](https://webassembly.github.io/spec/core/binary/modules.html#table-section).
#[derive(Clone, Copy)]
pub struct TablesComponent<B: Bytes> {
    types: Vector<u64, B>,
}

impl<B: Bytes> From<Vector<u64, B>> for TablesComponent<B> {
    #[inline]
    fn from(types: Vector<u64, B>) -> Self {
        Self { types }
    }
}

impl<B: Bytes> TablesComponent<B> {
    /// Uses the given [`Bytes`] to read the contents of the *table section* of a module, starting,
    /// at the specified `offset`.
    pub fn new(offset: u64, bytes: B) -> Result<Self> {
        Vector::parse(offset, bytes)
            .context("at start of table section")
            .map(Self::from)
    }

    /// Gets the expected remaining number of entries in the *table section* that have yet to be
    /// parsed.
    #[inline]
    pub fn remaining_count(&self) -> u32 {
        self.types.remaining_count()
    }

    pub(crate) fn borrowed(&self) -> TablesComponent<&B> {
        TablesComponent {
            types: self.types.borrowed(),
        }
    }
}

impl<B: Bytes> core::iter::Iterator for TablesComponent<B> {
    type Item = Result<TableType>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.types.advance(|offset, bytes| {
            crate::component::table_type(offset, bytes).context("within table section")
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.types.size_hint()
    }
}

impl<B: Clone + Bytes> core::iter::FusedIterator for TablesComponent<B> {}

impl<B: Bytes> core::fmt::Debug for TablesComponent<B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(&self.types, f)
    }
}
