use crate::{
    input::Input,
    parser::{self, Offset, ResultExt as _},
};

/// Helper struct for parsing a sequence of elements prefixed by a `u32` count, known in the
/// WebAssembly format as a
/// [`vec` or vector](https://webassembly.github.io/spec/core/binary/conventions.html#vectors).
#[derive(Clone, Copy)]
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
    pub fn parse(mut offset: O, input: I) -> parser::Result<Self> {
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

    /// Parses an element with the given closure, passing the number of items that have been parsed
    /// so far as the first parameter.
    ///
    /// # Errors
    ///
    /// See [`Vector::advance`] for more information.
    pub fn advance_with_index<'a, T, E, F>(&'a mut self, f: F) -> Option<Result<T, E>>
    where
        F: FnOnce(u32, &'a mut u64, &'a I) -> Result<T, E>,
    {
        if self.remaining == 0 {
            return None;
        }

        let result = f(
            self.total - self.remaining,
            self.offset.offset_mut(),
            &self.input,
        );

        if result.is_ok() {
            self.remaining -= 1;
        } else {
            self.remaining = 0;
        }

        Some(result)
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

    /// Returns a clone of the [`Vector`] by borrowing the underlying [`Input`].
    pub fn borrowed(&self) -> Vector<u64, &I> {
        Vector {
            total: self.total,
            remaining: self.remaining,
            offset: self.offset.offset(),
            input: &self.input,
        }
    }

    #[inline]
    pub(crate) fn input(&self) -> &I {
        &self.input
    }

    #[inline]
    pub(crate) fn into_offset(self) -> O {
        self.offset
    }
}

impl<O: Offset, I: Clone + Input> Vector<O, &I> {
    pub(crate) fn dereferenced(&self) -> Vector<u64, I> {
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
