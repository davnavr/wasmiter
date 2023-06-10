use crate::{
    bytes::Bytes,
    component::IndexOrder,
    custom::name::NameAssoc,
    index::Index,
    parser::{Offset, Result, ResultExt as _, Vector},
};

/// A [*name map*](https://webassembly.github.io/spec/core/appendix/custom.html#name-maps)
/// is a sequence of [`NameAssoc`] entries, which associate indices with names.
///
/// Each index is checked in order to ensure they are unique and in increasing order.
#[derive(Clone, Copy)]
pub struct NameMap<I: Index, O: Offset, B: Bytes> {
    order: IndexOrder<I>,
    entries: Vector<O, B>,
}

impl<I: Index, O: Offset, B: Bytes> From<Vector<O, B>> for NameMap<I, O, B> {
    fn from(entries: Vector<O, B>) -> Self {
        Self {
            order: IndexOrder::new(),
            entries,
        }
    }
}

impl<I: Index, O: Offset, B: Bytes> NameMap<I, O, B> {
    /// Creates a new [`NameMap`] to parse at the given `offset`.
    pub fn new(offset: O, bytes: B) -> Result<Self> {
        Vector::parse(offset, bytes).map(Self::from)
    }

    /// Parses the next entry in the [`NameMap`].
    pub fn parse(&mut self) -> Result<Option<NameAssoc<I, &B>>> {
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

    fn borrowed(&self) -> NameMap<I, u64, &B> {
        NameMap {
            order: self.order,
            entries: self.entries.borrowed(),
        }
    }
}

impl<I: Index, O: Offset, B: Clone + Bytes> Iterator for NameMap<I, O, B> {
    type Item = Result<NameAssoc<I, B>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.parse() {
            Ok(None) => None,
            Err(e) => Some(Err(e)),
            Ok(Some(name_assoc)) => Some(Ok(name_assoc.dereferenced())),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.entries.size_hint()
    }
}

impl<I: Index, O: Offset, B: Clone + Bytes> core::iter::FusedIterator for NameMap<I, O, B> {}

impl<I: Index, O: Offset, B: Bytes> core::fmt::Debug for NameMap<I, O, B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.borrowed()).finish()
    }
}
