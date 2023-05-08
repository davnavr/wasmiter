use crate::component::TableType;
use crate::parser::{input::Input, Decoder, Result, ResultExt, SimpleParse, Vector};

/// Represents the
/// [**tables** component](https://webassembly.github.io/spec/core/syntax/modules.html#tables) of a
/// WebAssembly module, stored in and parsed from the
/// [*tables section*](https://webassembly.github.io/spec/core/binary/modules.html#table-section).
pub struct TablesComponent<I: Input> {
    types: Vector<I, SimpleParse<TableType>>,
}

impl<I: Input> From<Vector<I, SimpleParse<TableType>>> for TablesComponent<I> {
    #[inline]
    fn from(types: Vector<I, SimpleParse<TableType>>) -> Self {
        Self { types }
    }
}

impl<I: Input> TablesComponent<I> {
    /// Uses the given [`Decoder<I>`] to read the contents of the *table section* of a module.
    pub fn new(input: Decoder<I>) -> Result<Self> {
        Vector::new(input, Default::default())
            .map(Self::from)
            .context("table section")
    }
}

impl<I: Input> core::iter::Iterator for TablesComponent<I> {
    type Item = Result<TableType>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.types.next().map(|r| r.context("table section"))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.types.size_hint()
    }
}

impl<I: Input> core::fmt::Debug for TablesComponent<I> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        crate::component::debug_section_contents(self.types.try_clone(), f)
    }
}
