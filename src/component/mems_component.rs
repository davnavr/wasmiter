use crate::{
    input::Input,
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
pub struct MemsComponent<I: Input> {
    types: Vector<u64, I>,
}

impl<I: Input> From<Vector<u64, I>> for MemsComponent<I> {
    #[inline]
    fn from(types: Vector<u64, I>) -> Self {
        Self { types }
    }
}

impl<I: Input> MemsComponent<I> {
    /// Uses the given [`Input`] to read the contents of the *memory section* of a module, starting
    /// at the specified `offset`.
    pub fn new(offset: u64, input: I) -> Result<Self> {
        Vector::parse(offset, input)
            .context("at start of memory section")
            .map(Self::from)
    }

    /// Gets the expected remaining number of entries in the *memory section* that have yet to be
    /// parsed.
    #[inline]
    pub fn remaining_count(&self) -> u32 {
        self.types.remaining_count()
    }

    pub(crate) fn borrowed(&self) -> MemsComponent<&I> {
        MemsComponent {
            types: self.types.borrowed(),
        }
    }
}

impl<I: Input> core::iter::Iterator for MemsComponent<I> {
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

impl<I: Clone + Input> core::iter::FusedIterator for MemsComponent<I> {}

impl<I: Input> core::fmt::Debug for MemsComponent<I> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(&self.types, f)
    }
}
