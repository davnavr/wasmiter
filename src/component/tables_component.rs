use crate::{
    input::Input,
    parser::{Result, ResultExt, Vector},
    types::TableType,
};

/// Represents the
/// [**tables** component](https://webassembly.github.io/spec/core/syntax/modules.html#tables) of a
/// WebAssembly module, stored in and parsed from the
/// [*tables section*](https://webassembly.github.io/spec/core/binary/modules.html#table-section).
#[derive(Clone, Copy)]
pub struct TablesComponent<I: Input> {
    types: Vector<u64, I>,
}

impl<I: Input> From<Vector<u64, I>> for TablesComponent<I> {
    #[inline]
    fn from(types: Vector<u64, I>) -> Self {
        Self { types }
    }
}

impl<I: Input> TablesComponent<I> {
    /// Uses the given [`Input`] to read the contents of the *table section* of a module, starting,
    /// at the specified `offset`.
    pub fn new(offset: u64, input: I) -> Result<Self> {
        Vector::parse(offset, input)
            .context("at start of table section")
            .map(Self::from)
    }

    /// Gets the expected remaining number of entries in the *table section* that have yet to be
    /// parsed.
    #[inline]
    pub fn remaining_count(&self) -> u32 {
        self.types.remaining_count()
    }

    pub(crate) fn borrowed(&self) -> TablesComponent<&I> {
        TablesComponent {
            types: self.types.borrowed(),
        }
    }
}

impl<I: Input> core::iter::Iterator for TablesComponent<I> {
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

impl<I: Clone + Input> core::iter::FusedIterator for TablesComponent<I> {}

impl<I: Input> core::fmt::Debug for TablesComponent<I> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(&self.types, f)
    }
}
