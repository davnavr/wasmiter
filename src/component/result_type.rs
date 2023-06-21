use crate::{
    input::{BorrowInput, CloneInput, HasInput, Input},
    parser::{Offset, Parsed, Vector},
    types::ValType,
};

/// Represents a
/// [WebAssembly result type](https://webassembly.github.io/spec/core/binary/types.html#result-types),
/// which is simply a [`Vector`] of [`ValType`]s.
#[derive(Clone, Copy)]
pub struct ResultType<O: Offset, I: Input> {
    types: Vector<O, I>,
}

impl<O: Offset, I: Input> From<Vector<O, I>> for ResultType<O, I> {
    #[inline]
    fn from(types: Vector<O, I>) -> Self {
        Self { types }
    }
}

impl<O: Offset, I: Input> ResultType<O, I> {
    pub(crate) fn empty_with_offset(offset: O, input: I) -> Self {
        Vector::new(0, offset, input).into()
    }

    /// Parses the start of a [`ResultType`].
    pub fn parse(offset: O, input: I) -> Parsed<Self> {
        Vector::parse(offset, input).map(Self::from)
    }

    /// Gets the remaining number of types.
    #[inline]
    pub fn remaining_count(&self) -> u32 {
        self.types.remaining_count()
    }

    /// Parses the remaining types.
    pub fn finish(mut self) -> Parsed<(O, I)> {
        for result in &mut self {
            let _ = result?;
        }

        Ok(self.types.into_offset_and_input())
    }
}

impl<O: Offset, I: Input> HasInput<I> for ResultType<O, I> {
    #[inline]
    fn input(&self) -> &I {
        self.types.input()
    }
}

impl<'a, O: Offset, I: Input + 'a> BorrowInput<'a, I> for ResultType<O, I> {
    type Borrowed = ResultType<u64, &'a I>;

    #[inline]
    fn borrow_input(&'a self) -> Self::Borrowed {
        self.types.borrow_input().into()
    }
}

impl<'a, O: Offset, I: Clone + Input + 'a> CloneInput<'a, I> for ResultType<O, &'a I> {
    type Cloned = ResultType<u64, I>;

    #[inline]
    fn clone_input(&self) -> Self::Cloned {
        self.types.clone_input().into()
    }
}

impl<O: Offset, I: Input> Iterator for ResultType<O, I> {
    type Item = Parsed<ValType>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.types.advance(crate::component::val_type)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.types.size_hint()
    }
}

impl<O: Offset, I: Input> core::fmt::Debug for ResultType<O, I> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.borrow_input()).finish()
    }
}
