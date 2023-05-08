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
        Self {
            count,
            parser,
        }
    }

    /// Gets the remaining number of elements in the sequence.
    pub fn count(&self) -> u32 {
        self.count
    }

    /// Parses the remaining elements in the sequence, discarding the results.
    pub fn finish<I: Input>(mut self, input: &mut Decoder<I>) -> Result<()> {
        while let Some(_) = self.parse(input)? {}
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
    pub fn vector<P: Parse, F: FnMut(P::Output) -> bool>(&mut self, parser: P, mut f: F) -> Result<()> {
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
