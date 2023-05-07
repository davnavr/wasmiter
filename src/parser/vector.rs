use crate::parser::{input::Input, Decoder, Result};

use super::ResultExt;

/// Parses a sequence of elements prefixed by a `u32` length.
pub struct Vector<I: Input, P> {
    count: u32,
    source: Decoder<I>,
    parser: P,
}

impl<I: Input, P> Vector<I, P> {
    pub fn new(mut source: Decoder<I>, parser: P) -> Result<Self> {
        Ok(Self {
            count: source.leb128_u32().context("vector length")?,
            source,
            parser,
        })
    }
}

impl<I: Input> Decoder<I> {
    //P: FnMut(&mut Parser<&mut I>) -> T
    pub fn vector<P>(&mut self, parser: P) -> Result<Vector<&mut I, P>> {
        todo!()
    }
}
