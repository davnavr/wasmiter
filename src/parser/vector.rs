use crate::bytes::Bytes;
use crate::parser::{self, Offset, Parse, Result, ResultExt};

/// Parser for a sequence of elements.
#[derive(Clone, Copy, Debug, Default)]
#[must_use]
pub struct Sequence<P: Parse> {
    count: u32,
    parser: P,
}

impl<P: Parse> Sequence<P> {
    /// Creates a new `Sequence` with the given `count`.
    pub const fn new(count: u32, parser: P) -> Self {
        Self { count, parser }
    }

    /// Gets the remaining number of elements in the sequence.
    #[inline]
    pub fn len(&self) -> u32 {
        self.count
    }

    /// Returns a value indicating if sequence of elements is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Parses the remaining elements in the sequence, discarding the results.
    pub fn finish<B: Bytes>(mut self, offset: &mut u64, bytes: B) -> Result<()> {
        while self.parse(offset, &bytes)?.is_some() {}
        Ok(())
    }
}

impl<P: Parse> Parse for Sequence<P> {
    type Output = Option<P::Output>;

    fn parse<B: Bytes>(&mut self, offset: &mut u64, bytes: B) -> Result<Self::Output> {
        if self.count == 0 {
            return Ok(None);
        }

        let result = self.parser.parse(offset, bytes);

        if result.is_ok() {
            self.count -= 1;
        } else {
            self.count = 0;
        }

        result.map(Some)
    }
}

/// Represents a sequence of elements prefixed by a `u32` count.
#[derive(Clone, Copy)]
pub struct Vector<O: Offset, B: Bytes, P: Parse> {
    offset: O,
    bytes: B,
    sequence: Sequence<P>,
}

impl<'a, O: Offset, B: Bytes, P: Parse> Vector<O, &'a B, P> {
    #[inline]
    pub(crate) const fn empty_with_offset(offset: O, bytes: &'a B, parser: P) -> Self {
        Self {
            offset,
            bytes,
            sequence: Sequence::new(0, parser),
        }
    }
}

impl<'a, B: Bytes, P: Parse> Vector<u64, &'a B, P> {
    #[inline]
    pub(crate) const fn empty(bytes: &'a B, parser: P) -> Self {
        Self::empty_with_offset(0, bytes, parser)
    }
}

impl<O: Offset, B: Bytes, P: Parse> Vector<O, B, P> {
    /// Creates a new [`Vector`] from the given [`Bytes`].
    pub fn new(mut offset: O, bytes: B, parser: P) -> Result<Self> {
        let count =
            parser::leb128::u32(offset.offset_mut(), &bytes).context("vector element count")?;
        Ok(Self {
            offset,
            bytes,
            sequence: Sequence::new(count, parser),
        })
    }

    /// Gets the remaining number of elements in the vector.
    #[inline]
    pub fn len(&self) -> u32 {
        self.sequence.len()
    }

    /// Returns a value indicating if the vector does not have any elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.sequence.is_empty()
    }

    /// Parses the remaining elements in the vector, discarding the results.
    pub fn finish(mut self) -> Result<O> {
        self.sequence.finish(self.offset.offset_mut(), self.bytes)?;
        Ok(self.offset)
    }
}

impl<O: Offset, B: Bytes, P: Parse + Clone> Vector<O, B, P> {
    pub(crate) fn by_reference(&self) -> Vector<u64, &B, P> {
        Vector {
            offset: self.offset.offset(),
            bytes: &self.bytes,
            sequence: self.sequence.clone(),
        }
    }
}

impl<'a, O: Offset, B: Bytes, P: Parse + Clone> Vector<O, &&'a B, P> {
    pub(crate) fn dereferenced(&self) -> Vector<u64, &'a B, P> {
        Vector {
            offset: self.offset.offset(),
            bytes: self.bytes,
            sequence: self.sequence.clone(),
        }
    }
}

impl<O: Offset, B: Bytes, P: Parse> Iterator for Vector<O, B, P> {
    type Item = Result<P::Output>;

    fn next(&mut self) -> Option<Self::Item> {
        self.sequence
            .parse(self.offset.offset_mut(), &self.bytes)
            .transpose()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (
            core::cmp::min(1, usize::try_from(self.sequence.count).unwrap_or(1)),
            usize::try_from(self.sequence.count).ok(),
        )
    }
}

impl<O, B, P> core::fmt::Debug for Vector<O, B, P>
where
    O: Offset,
    B: Bytes,
    P: Parse + Clone,
    P::Output: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.by_reference()).finish()
    }
}

/// Parses a sequence of elements prefixed by a `u32` length.
pub fn vector<P: Parse, B: Bytes, F: FnMut(P::Output) -> bool>(
    offset: &mut u64,
    bytes: B,
    parser: P,
    mut f: F,
) -> Result<()> {
    let count = parser::leb128::u32(offset, &bytes).context("vector length")?;
    let mut sequence = Sequence::new(count, parser);
    while let Some(item) = sequence.parse(offset, &bytes)? {
        if !f(item) {
            break;
        }
    }
    Ok(())
}
