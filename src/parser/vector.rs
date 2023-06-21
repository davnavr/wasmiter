use crate::{
    input::{self, Input},
    parser::{self, Offset, ResultExt as _},
};

/// Helper struct for parsing a sequence of elements prefixed by a `u32` count, known in the
/// WebAssembly format as a
/// [`vec` or vector](https://webassembly.github.io/spec/core/binary/conventions.html#vectors).
#[derive(Clone, Copy)]
#[must_use]
pub struct Vector<O: Offset, I: Input> {
    total: u32,
    remaining: u32,
    offset: O,
    input: I,
}

impl<O: Offset, I: Input> Vector<O, I> {
    /// Constructs a new [`Vector`] with the given `count`, whose elements start at the given
    /// `offset` into the [`Input`].
    pub fn new(count: u32, offset: O, input: I) -> Self {
        Self {
            total: count,
            remaining: count,
            offset,
            input,
        }
    }

    /// Parses the given [`Input`] to obtain the `u32` count of elements.
    pub fn parse(mut offset: O, input: I) -> parser::Parsed<Self> {
        let count =
            parser::leb128::u32(offset.offset_mut(), &input).context("vector element count")?;

        Ok(Self::new(count, offset, input))
    }

    /// Gets the expected remaining number of elements in the [`Vector`].
    #[inline]
    pub fn remaining_count(&self) -> u32 {
        self.remaining
    }

    /// Implementation of [`Iterator::size_hint`].
    pub fn size_hint(&self) -> (usize, Option<usize>) {
        (
            usize::from(self.remaining != 0),
            usize::try_from(self.remaining).ok(),
        )
    }

    /// Returns the [`Offset`] into the next element of the [`Vector`], and the [`Input`] elements
    /// were read from.
    #[inline]
    pub fn into_offset_and_input(self) -> (O, I) {
        (self.offset, self.input)
    }

    /// Parses an element with the given closure, passing the element's index as the first
    /// parameter.
    ///
    /// # Errors
    ///
    /// See the documentation for [`Vector::advance`] for more information.
    pub fn advance_with_index<'a, T, E, F>(&'a mut self, f: F) -> Option<Result<T, E>>
    where
        F: FnOnce(u32, &'a mut u64, &'a I) -> Result<T, E>,
    {
        if self.remaining == 0 {
            None
        } else {
            let result = f(
                self.total - self.remaining,
                self.offset.offset_mut(),
                &self.input,
            );

            if result.is_ok() {
                self.remaining -= 1;
            } else {
                self.remaining = 0;
            };

            Some(result)
        }
    }

    /// Parses an element with the given closure.
    ///
    /// # Errors
    ///
    /// Returns any errors returned by the closure. If an error was returned, then future calls to
    /// [`advance`](Vector::advance) will return `None`.
    #[inline]
    pub fn advance<'a, T, E, F>(&'a mut self, f: F) -> Option<Result<T, E>>
    where
        F: FnOnce(&'a mut u64, &'a I) -> Result<T, E>,
    {
        self.advance_with_index(|_, offset, bytes| f(offset, bytes))
    }

    /// Parses all of the remaining elements in the vector with the given closure, passing the
    /// element's index as the first parameter.
    ///
    /// # Errors
    ///
    /// See the documentation for [`Vector::advance`] for more information.
    pub fn finish_with_index<E, F>(self, mut f: F) -> Result<(O, I), E>
    where
        F: FnMut(u32, &mut u64, &I) -> Result<(), E>,
    {
        let Self {
            mut offset,
            input,
            remaining,
            ..
        } = self;

        for index in 0..remaining {
            f(index, offset.offset_mut(), &input)?;
        }

        Ok((offset, input))
    }

    /// Parses all of the remaining elements in the vector with the given closure.
    ///
    /// If the closure requires the index of the element, use [`Vector::finish_with_index`]
    /// instead.
    ///
    /// # Errors
    ///
    /// See the documentation for [`Vector::advance`] for more information.
    #[inline]
    pub fn finish<E, F>(self, mut f: F) -> Result<(O, I), E>
    where
        F: FnMut(&mut u64, &I) -> Result<(), E>,
    {
        self.finish_with_index(|_, offset, input| f(offset, input))
    }
}

impl<O: Offset, I: Input> input::HasInput<I> for Vector<O, I> {
    #[inline]
    fn input(&self) -> &I {
        &self.input
    }
}

impl<'a, O: Offset, I: Input + 'a> input::BorrowInput<'a, I> for Vector<O, I> {
    type Borrowed = Vector<u64, &'a I>;

    #[inline]
    fn borrow_input(&'a self) -> Self::Borrowed {
        Vector {
            total: self.total,
            remaining: self.remaining,
            offset: self.offset.offset(),
            input: &self.input,
        }
    }
}

impl<'a, O: Offset, I: Clone + Input + 'a> input::CloneInput<'a, I> for Vector<O, &'a I> {
    type Cloned = Vector<u64, I>;

    #[inline]
    fn clone_input(&self) -> Self::Cloned {
        Vector {
            total: self.total,
            remaining: self.remaining,
            offset: self.offset.offset(),
            input: self.input.clone(),
        }
    }
}

impl<O: Offset, I: Input> core::fmt::Debug for Vector<O, I> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Vector")
            .field("remaining", &self.remaining)
            .field("offset", &self.offset.offset())
            .finish_non_exhaustive()
    }
}
