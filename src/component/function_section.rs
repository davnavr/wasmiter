use crate::component::FuncIdx;
use crate::parser::{input::Input, Parser, Result, ResultExt};

/// Represents the
/// [*function section*](https://webassembly.github.io/spec/core/binary/modules.html#function-section).
pub struct FunctionSection<I: Input> {
    count: usize,
    parser: Parser<I>,
}
impl<I: Input> FunctionSection<I> {
    /// Uses the given [`Parser<I>`] to read the contents of the *function section* of a module.
    pub fn new(mut parser: Parser<I>) -> Result<Self> {
        Ok(Self {
            count: parser.leb128_usize().context("function section count")?,
            parser,
        })
    }
}

impl<I: Input> core::iter::Iterator for FunctionSection<I> {
    type Item = Result<FuncIdx>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count == 0 {
            return None;
        }

        let result = self.parser.index().context("index in function section");
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
impl<I: Input> core::iter::ExactSizeIterator for FunctionSection<I> {
    fn len(&self) -> usize {
        self.count
    }
}

impl<I: Input> core::fmt::Debug for FunctionSection<I> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.parser.fork() {
            Ok(fork) => f
                .debug_list()
                .entries(FunctionSection {
                    count: self.count,
                    parser: fork,
                })
                .finish(),
            Err(failed) => f
                .debug_list()
                .entries(core::iter::once(Result::<()>::Err(failed)))
                .finish(),
        }
    }
}
