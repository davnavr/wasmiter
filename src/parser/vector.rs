use crate::parser::{input::Input, Decoder, Parse, Result};

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
    pub fn len(&self) -> u32 {
        self.count
    }

    /// Parses the remaining elements in the sequence, discarding the results.
    pub fn finish<I: Input>(mut self, input: &mut Decoder<I>) -> Result<()> {
        while self.parse(input)?.is_some() {}
        Ok(())
    }
}

impl<P: Parse> Parse for Sequence<P> {
    type Output = Option<P::Output>;

    fn parse<I: Input>(&mut self, input: &mut Decoder<I>) -> Result<Self::Output> {
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

impl<I: Input> Decoder<I> {
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
pub struct Vector<I: Input, P: Parse> {
    decoder: Decoder<I>,
    sequence: Sequence<P>,
}

impl<I: Input, P: Parse> Vector<I, P> {
    /// Creates a new [`Vector`] from the given `input`.
    pub fn new(mut input: Decoder<I>, parser: P) -> Result<Self> {
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

    /// Parses the remaining elements in the vector, discarding the results.
    pub fn finish(mut self) -> Result<()> {
        self.sequence.finish(&mut self.decoder)
    }

    fn try_clone(&self) -> Result<Vector<I::Fork, P>> {
        Ok(Vector {
            decoder: self.decoder.fork()?,
            sequence: self.sequence.clone(),
        })
    }
}

impl<I: Input, P: Parse> Iterator for Vector<I, P> {
    type Item = Result<P::Output>;

    fn next(&mut self) -> Option<Self::Item> {
        self.sequence.parse(&mut self.decoder).transpose()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, usize::try_from(self.sequence.count).ok())
    }
}

impl<I: Input, P: Parse> core::fmt::Debug for Vector<I, P> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Vector")
            .field("count", &self.len())
            .finish_non_exhaustive()
    }
}
