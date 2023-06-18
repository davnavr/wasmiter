use crate::{
    component,
    index::Index,
    input::{BorrowInput, HasInput, Input},
    parser::{Offset, Result, Vector},
};

/// Represents a [`Vector`] of WebAssembly indices.
#[derive(Clone, Copy)]
pub struct IndexVector<N: Index, O: Offset, I: Input> {
    indices: Vector<O, I>,
    _marker: core::marker::PhantomData<&'static N>,
}

impl<N: Index, O: Offset, I: Input> IndexVector<N, O, I> {
    /// Creates a new [`IndexVector`] with the given `count`, and whose elements start in the
    /// [`Input`] at the given `offset`.
    #[inline]
    pub fn new(count: u32, offset: O, input: I) -> Self {
        Vector::new(count, offset, input).into()
    }

    /// Creates a new [`IndexVector`] with a parsed `u32` count from the given [`Input`].
    #[inline]
    pub fn parse(offset: O, input: I) -> Result<Self> {
        Vector::parse(offset, input).map(Self::from)
    }

    /// Gets the remaining number of indices.
    #[inline]
    pub fn remaining_count(&self) -> u32 {
        self.indices.remaining_count()
    }

    /// Parses the remaining indices.
    pub fn finish(mut self) -> Result<O> {
        for result in &mut self {
            let _ = result?;
        }

        Ok(self.indices.into_offset())
    }
}

impl<N: Index, O: Offset, I: Input> HasInput<I> for IndexVector<N, O, I> {
    #[inline]
    fn input(&self) -> &I {
        self.indices.input()
    }
}

impl<'a, N: Index, O: Offset, I: Input + 'a> BorrowInput<'a, I> for IndexVector<N, O, I> {
    type Borrowed = IndexVector<N, u64, &'a I>;

    #[inline]
    fn borrow_input(&'a self) -> Self::Borrowed {
        self.indices.borrow_input().into()
    }
}

impl<N: Index, O: Offset, I: Input> From<Vector<O, I>> for IndexVector<N, O, I> {
    #[inline]
    fn from(indices: Vector<O, I>) -> Self {
        Self {
            indices,
            _marker: core::marker::PhantomData,
        }
    }
}

impl<N: Index, O: Offset, I: Input> Iterator for IndexVector<N, O, I> {
    type Item = Result<N>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.indices.advance(component::index)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.indices.size_hint()
    }
}

impl<N: Index, O: Offset, I: Input> core::iter::FusedIterator for IndexVector<N, O, I> {}

impl<N: Index, O: Offset, I: Input> core::fmt::Debug for IndexVector<N, O, I> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.borrow_input()).finish()
    }
}
