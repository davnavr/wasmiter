use crate::component::GlobalType;
use crate::parser::{input::Input, Decoder, Result, ResultExt};

/// Represents a
/// [WebAssembly global](https://webassembly.github.io/spec/core/syntax/modules.html#globals).
#[derive(Debug)]
pub struct Global {
    r#type: GlobalType,
    //expression: Expression<I>,
}

impl Global {
    /// Gets the type of the value stored in the global.
    pub fn value_type(&self) -> &GlobalType {
        &self.r#type
    }
}

/// Represents the
/// [**globals** component](https://webassembly.github.io/spec/core/syntax/modules.html#globals) of
/// a WebAssembly module, stored in and parsed from the
/// [*global section*](https://webassembly.github.io/spec/core/binary/modules.html#global-section).
pub struct GlobalsComponent<I: Input> {
    count: usize,
    parser: Decoder<I>,
}

impl<I: Input> GlobalsComponent<I> {
    /// Uses the given [`Decoder<I>`] to read the contents of the *global section* of a module.
    pub fn new(mut parser: Decoder<I>) -> Result<Self> {
        Ok(Self {
            count: parser.leb128_usize().context("global section count")?,
            parser,
        })
    }

    fn try_clone(&self) -> Result<GlobalsComponent<I::Fork>> {
        Ok(GlobalsComponent {
            count: self.count,
            parser: self.parser.fork()?,
        })
    }
}

impl<I: Input> core::iter::Iterator for GlobalsComponent<I> {
    type Item = Result<Global>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count == 0 {
            return None;
        }

        let global = todo!("parse expr");

        self.count -= 1;
        Some(global)
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
impl<I: Input> core::iter::ExactSizeIterator for GlobalsComponent<I> {
    fn len(&self) -> usize {
        self.count
    }
}

impl<I: Input> core::fmt::Debug for GlobalsComponent<I> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        crate::component::debug_section_contents(self.try_clone(), f)
    }
}
