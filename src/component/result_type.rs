use crate::{
    bytes::Bytes,
    parser::{Offset, Result, Vector},
    types::ValType,
};

/// Represents a
/// [WebAssembly result type](https://webassembly.github.io/spec/core/binary/types.html#result-types),
/// which is simply a [`Vector`] of [`ValType`]s.
#[derive(Clone, Copy)]
pub struct ResultType<O: Offset, B: Bytes> {
    types: Vector<O, B>,
}

impl<O: Offset, B: Bytes> From<Vector<O, B>> for ResultType<O, B> {
    #[inline]
    fn from(types: Vector<O, B>) -> Self {
        Self { types }
    }
}

impl<O: Offset, B: Bytes> ResultType<O, B> {
    pub(crate) fn empty_with_offset(offset: O, bytes: B) -> Self {
        Vector::new(0, offset, bytes).into()
    }

    /// Parses the start of a [`ResultType`].
    pub fn parse(offset: O, bytes: B) -> Result<Self> {
        Vector::parse(offset, bytes).map(Self::from)
    }

    /// Returns a clone of the [`ResultType`], borrowing the underlying [`Bytes`].
    #[inline]
    pub fn borrowed(&self) -> ResultType<u64, &B> {
        self.types.borrowed().into()
    }

    /// Gets the remaining number of types.
    #[inline]
    pub fn remaining_count(&self) -> u32 {
        self.types.remaining_count()
    }

    /// Parses the remaining types.
    pub fn finish(mut self) -> Result<O> {
        for result in &mut self {
            let _ = result?;
        }

        Ok(self.types.into_offset())
    }
}

impl<O: Offset, B: Clone + Bytes> ResultType<O, &B> {
    #[inline]
    pub(crate) fn dereferenced(&self) -> ResultType<u64, B> {
        self.types.dereferenced().into()
    }
}

impl<O: Offset, B: Bytes> Iterator for ResultType<O, B> {
    type Item = Result<ValType>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.types.advance(crate::component::val_type)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.types.size_hint()
    }
}

impl<O: Offset, B: Bytes> core::fmt::Debug for ResultType<O, B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.borrowed()).finish()
    }
}
