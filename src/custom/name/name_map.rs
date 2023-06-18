use crate::{
    custom::name::NameAssoc,
    index::Index,
    input::{BorrowInput, CloneInput, HasInput, Input},
    parser::{AscendingOrder, Offset, Result, ResultExt as _, Vector},
};

/// A [*name map*](https://webassembly.github.io/spec/core/appendix/custom.html#name-maps) is a
/// sequence of [`NameAssoc`] entries, which associate indices with names.
///
/// Each index is checked in order to ensure they are unique and in increasing order.
#[derive(Clone, Copy)]
pub struct NameMap<N: Index, O: Offset, I: Input> {
    order: AscendingOrder<u32, N>,
    entries: Vector<O, I>,
}

impl<N: Index, O: Offset, I: Input> From<Vector<O, I>> for NameMap<N, O, I> {
    fn from(entries: Vector<O, I>) -> Self {
        Self {
            order: AscendingOrder::new(),
            entries,
        }
    }
}

impl<N: Index, O: Offset, I: Input> NameMap<N, O, I> {
    /// Parses a [`NameMap`] starting at the given `offset`.
    pub fn new(offset: O, input: I) -> Result<Self> {
        Vector::parse(offset, input).map(Self::from)
    }

    /// Gets the remaining number of entries in the [`NameMap`].
    #[inline]
    pub fn remaining_count(&self) -> u32 {
        self.entries.remaining_count()
    }

    /// Parses the next entry in the [`NameMap`].
    pub fn parse(&mut self) -> Result<Option<NameAssoc<N, &I>>> {
        self.entries
            .advance_with_index(|i, offset, bytes| {
                let name_assoc =
                    NameAssoc::parse(offset, bytes).context("while parsing name map entry")?;

                self.order
                    .check(name_assoc.index(), i == 0)
                    .context("name map index was invalid")?;

                Ok(name_assoc)
            })
            .transpose()
    }

    /// Parses all remaining entries in the [`NameMap`].
    pub fn finish(mut self) -> Result<O> {
        while self.parse()?.is_some() {}
        Ok(self.entries.into_offset())
    }
}

impl<N: Index, O: Offset, I: Input> HasInput<I> for NameMap<N, O, I> {
    #[inline]
    fn input(&self) -> &I {
        self.entries.input()
    }
}

impl<'a, N: Index, O: Offset, I: Input + 'a> BorrowInput<'a, I> for NameMap<N, O, I> {
    type Borrowed = NameMap<N, u64, &'a I>;

    #[inline]
    fn borrow_input(&'a self) -> Self::Borrowed {
        NameMap {
            order: self.order,
            entries: self.entries.borrow_input(),
        }
    }
}

impl<'a, N: Index, O: Offset, I: Clone + Input + 'a> CloneInput<'a, I> for NameMap<N, O, &'a I> {
    type Cloned = NameMap<N, u64, I>;

    #[inline]
    fn clone_input(&self) -> Self::Cloned {
        NameMap {
            order: self.order,
            entries: self.entries.clone_input(),
        }
    }
}

impl<N: Index, O: Offset, I: Clone + Input> NameMap<N, O, &I> {}

impl<N: Index, O: Offset, I: Clone + Input> Iterator for NameMap<N, O, I> {
    type Item = Result<NameAssoc<N, I>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.parse() {
            Ok(None) => None,
            Err(e) => Some(Err(e)),
            Ok(Some(name_assoc)) => Some(Ok(name_assoc.clone_input())),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.entries.size_hint()
    }
}

impl<N: Index, O: Offset, I: Clone + Input> core::iter::FusedIterator for NameMap<N, O, I> {}

impl<N: Index, O: Offset, I: Input> core::fmt::Debug for NameMap<N, O, I> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.borrow_input()).finish()
    }
}
