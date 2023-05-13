use crate::parser::{input::Bytes, Decoder, Parse, Result};

use super::ResultExt;

/// Parser for a sequence of elements.
#[derive(Clone, Debug, Default)]
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
    pub fn finish<B: Bytes>(mut self, input: &mut Decoder<B>) -> Result<()> {
        while self.parse(input)?.is_some() {}
        Ok(())
    }
}

impl<P: Parse> Parse for Sequence<P> {
    type Output = Option<P::Output>;

    fn parse<B: Bytes>(&mut self, input: &mut Decoder<B>) -> Result<Self::Output> {
        if self.count == 0 {
            return Ok(None);
        }

        let result = self.parser.parse(input);

        if result.is_ok() {
            self.count -= 1;
        } else {
            self.count = 0;
        }

        result.map(Some)
    }
}

impl<B: Bytes> Decoder<B> {
    /// Parses a sequence of elements prefixed by a `u32` length.
    pub fn vector<P: Parse, F: FnMut(P::Output) -> bool>(
        &mut self,
        parser: P,
        mut f: F,
    ) -> Result<()> {
        let count = self.leb128_u32().context("vector length")?;
        let mut sequence = Sequence::new(count, parser);
        while let Some(item) = sequence.parse(self)? {
            if !f(item) {
                break;
            }
        }
        Ok(())
    }
}

/// Represents a sequence of elements prefixed by a `u32` count.
#[derive(Clone)]
pub struct Vector<B: Bytes, P: Parse> {
    decoder: Decoder<B>,
    sequence: Sequence<P>,
}

impl<B: Bytes, P: Parse> Vector<B, P> {
    /// Creates a new [`Vector`] from the given `input`.
    pub fn new(mut input: Decoder<B>, parser: P) -> Result<Self> {
        let count = input.leb128_u32().context("vector element count")?;
        Ok(Self {
            decoder: input,
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
    pub fn finish(mut self) -> Result<()> {
        self.sequence.finish(&mut self.decoder)
    }
}

impl<B: Bytes, P: Parse> Iterator for Vector<B, P> {
    type Item = Result<P::Output>;

    fn next(&mut self) -> Option<Self::Item> {
        self.sequence.parse(&mut self.decoder).transpose()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (
            core::cmp::min(1, usize::try_from(self.sequence.count).unwrap_or(1)),
            usize::try_from(self.sequence.count).ok(),
        )
    }
}

impl<B: Bytes + Clone, P: Parse + Clone> core::fmt::Debug for Vector<B, P> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(&self.clone()).finish()
    }
}
