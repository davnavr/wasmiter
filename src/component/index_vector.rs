use crate::{
    bytes::Bytes,
    component,
    index::Index,
    parser::{Offset, Result, Vector},
};

/// Represents a [`Vector`] of WebAssembly indices.
#[derive(Clone, Copy)]
pub struct IndexVector<I: Index, O: Offset, B: Bytes> {
    indices: Vector<O, B>,
    _marker: core::marker::PhantomData<&'static I>,
}

impl<I: Index, O: Offset, B: Bytes> IndexVector<I, O, B> {
    /// Creates a new [`IndexVector`] with the given `count`, and whose elements start at the given `offset`.
    #[inline]
    pub fn new(count: u32, offset: O, bytes: B) -> Self {
        Vector::new(count, offset, bytes).into()
    }

    /// Creates a new [`IndexVector`] with a parsed `u32` count from the given [`Bytes`].
    #[inline]
    pub fn parse(offset: O, bytes: B) -> Result<Self> {
        Vector::parse(offset, bytes).map(Self::from)
    }

    /// Gets the remaining number of indices.
    #[inline]
    pub fn remaining_count(&self) -> u32 {
        self.indices.remaining_count()
    }

    /// Returns a clone of the [`IndexVector`], borrowing the underlying [`Bytes`].
    #[inline]
    pub fn borrowed(&self) -> IndexVector<I, u64, &B> {
        self.indices.borrowed().into()
    }

    /// Parses the remaining indices.
    pub fn finish(mut self) -> Result<O> {
        for result in &mut self {
            let _ = result?;
        }

        Ok(self.indices.into_offset())
    }
}

impl<I: Index, O: Offset, B: Bytes> From<Vector<O, B>> for IndexVector<I, O, B> {
    #[inline]
    fn from(indices: Vector<O, B>) -> Self {
        Self {
            indices,
            _marker: core::marker::PhantomData,
        }
    }
}

impl<I: Index, O: Offset, B: Bytes> Iterator for IndexVector<I, O, B> {
    type Item = Result<I>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.indices.advance(component::index)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.indices.size_hint()
    }
}

impl<I: Index, O: Offset, B: Bytes> core::iter::FusedIterator for IndexVector<I, O, B> {}

impl<I: Index, O: Offset, B: Bytes> core::fmt::Debug for IndexVector<I, O, B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.borrowed()).finish()
    }
}
