use crate::component::TableType;
use crate::parser::{input::Input, Decoder, Result, ResultExt};

/// Represents the
/// [**tables** component](https://webassembly.github.io/spec/core/syntax/modules.html#tables) of a
/// WebAssembly module, stored in and parsed from the
/// [*tables section*](https://webassembly.github.io/spec/core/binary/modules.html#table-section).
pub struct TablesComponent<I: Input> {
    count: usize,
    parser: Decoder<I>,
}

impl<I: Input> TablesComponent<I> {
    /// Uses the given [`Decoder<I>`] to read the contents of the *table section* of a module.
    pub fn new(mut parser: Decoder<I>) -> Result<Self> {
        Ok(Self {
            count: parser.leb128_usize().context("table section count")?,
            parser,
        })
    }

    fn try_clone(&self) -> Result<TablesComponent<I::Fork>> {
        Ok(TablesComponent {
            count: self.count,
            parser: self.parser.fork()?,
        })
    }
}

impl<I: Input> core::iter::Iterator for TablesComponent<I> {
    type Item = Result<TableType>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count == 0 {
            return None;
        }

        let result = self.parser.table_type().context("table section");
        self.count -= 1;
        Some(result)
    }

    #[inline]
    fn count(self) -> usize {
        self.count
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.count, Some(self.count))
    }
}

// count is correct, since errors are returned if there are too few elements
impl<I: Input> core::iter::ExactSizeIterator for TablesComponent<I> {
    fn len(&self) -> usize {
        self.count
    }
}

impl<I: Input> core::fmt::Debug for TablesComponent<I> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        crate::component::debug_section_contents(self.try_clone(), f)
    }
}
