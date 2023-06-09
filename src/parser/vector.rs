use crate::{
    bytes::Bytes,
    parser::{self, Offset, ResultExt as _},
};

/// Helper struct for parsing a sequence of elements prefixed by a `u32` count, known in the
/// WebAssembly format as a
/// [`vec` or vector](https://webassembly.github.io/spec/core/binary/conventions.html#vectors).
#[derive(Clone, Copy)]
pub struct Vector<O: Offset, B: Bytes> {
    counter: u32,
    offset: O,
    bytes: B,
}

impl<O: Offset, B: Bytes> Vector<O, B> {
    /// Constructs a new [`Vector`] with the given `count`, whose elements start at the given
    /// `offset` into the `Bytes`.
    pub fn new(count: u32, offset: O, bytes: B) -> Self {
        Self {
            counter: count,
            offset,
            bytes,
        }
    }

    /// Parses the given [`Bytes`] to obtain the `u32` count of elements.
    pub fn parse(mut offset: O, bytes: B) -> parser::Result<Self> {
        Ok(Self {
            counter: parser::leb128::u32(offset.offset_mut(), &bytes)
                .context("vector element count")?,
            offset,
            bytes,
        })
    }

    /// Gets the expected remaining number of elements in the [`Vector`].
    #[inline]
    pub fn remaining_count(&self) -> u32 {
        self.counter
    }

    /// Implementation of [`Iterator::size_hint`].
    pub fn size_hint(&self) -> (usize, Option<usize>) {
        (
            usize::from(self.counter != 0),
            usize::try_from(self.counter).ok(),
        )
    }

    /// Parses an element with the given closure.
    ///
    /// # Errors
    ///
    /// Returns any errors returned by the closure. If an error was returned, then future calls to
    /// [`advance`](Vector::advance) will return `None`.
    pub fn advance<'a, T, E, F>(&'a mut self, f: F) -> Option<Result<T, E>>
    where
        F: FnOnce(&'a mut u64, &'a B) -> Result<T, E>,
    {
        if self.counter == 0 {
            return None;
        }

        let result = f(self.offset.offset_mut(), &self.bytes);

        if result.is_ok() {
            self.counter -= 1;
        } else {
            self.counter = 0;
        }

        Some(result)
    }

    /// Returns a clone of the [`Vector`] by borrowing the underlying [`Bytes`].
    pub fn borrowed(&self) -> Vector<u64, &B> {
        Vector {
            counter: self.counter,
            offset: self.offset.offset(),
            bytes: &self.bytes,
        }
    }

    #[inline]
    pub(crate) fn bytes(&self) -> &B {
        &self.bytes
    }

    #[inline]
    pub(crate) fn into_offset(self) -> O {
        self.offset
    }
}

impl<O: Offset, B: Clone + Bytes> Vector<O, &B> {
    pub(crate) fn dereferenced(&self) -> Vector<u64, B> {
        Vector {
            counter: self.counter,
            offset: self.offset.offset(),
            bytes: self.bytes.clone(),
        }
    }
}

impl<O: Offset, B: Bytes> core::fmt::Debug for Vector<O, B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Vector")
            .field("count", &self.counter)
            .field("offset", &self.offset.offset())
            .finish_non_exhaustive()
    }
}
