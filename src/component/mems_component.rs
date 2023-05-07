use crate::parser::{input::Input, Parser, Result, ResultExt};

/// Represents the
/// [**mems** component](https://webassembly.github.io/spec/core/syntax/modules.html#memories) of a
/// WebAssembly module, stored in and parsed from the
/// [*memory section*](https://webassembly.github.io/spec/core/binary/modules.html#memory-section).
pub struct MemsComponent<I: Input> {
    count: usize,
    parser: Parser<I>,
}

impl<I: Input> MemsComponent<I> {
    /// Uses the given [`Parser<I>`] to read the contents of the *memory section* of a module.
    pub fn new(mut parser: Parser<I>) -> Result<Self> {
        Ok(Self {
            count: parser.leb128_usize().context("memory section count")?,
            parser,
        })
    }

    fn try_clone(&self) -> Result<MemsComponent<I::Fork>> {
        Ok(MemsComponent {
            count: self.count,
            parser: self.parser.fork()?,
        })
    }
}

impl<I: Input> core::iter::Iterator for MemsComponent<I> {
    type Item = Result<crate::component::MemType>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count == 0 {
            return None;
        }

        let result = self.parser.mem_type().context("memory section");
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
impl<I: Input> core::iter::ExactSizeIterator for MemsComponent<I> {
    fn len(&self) -> usize {
        self.count
    }
}

impl<I: Input> core::fmt::Debug for MemsComponent<I> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        crate::component::debug_section_contents(self.try_clone(), f)
    }
}
