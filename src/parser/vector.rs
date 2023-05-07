use crate::parser::{input::Input, Parser, Result};

use super::ResultExt;

/// Parses a sequence of elements prefixed by a `u32` length.
pub struct Vector<I: Input, P> {
    count: u32,
    source: Parser<I>,
    parser: P,
}

impl<I: Input, P> Vector<I, P> {
    pub fn new(mut source: Parser<I>, parser: P) -> Result<Self> {
        Ok(Self {
            count: source.leb128_u32().context("vector length")?,
            source,
            parser,
        })
    }
}

impl<I: Input> Parser<I> {
    //P: FnMut(&mut Parser<&mut I>) -> T
    pub fn vector<P>(&mut self, parser: P) -> Result<Vector<&mut I, P>> {
        todo!()
    }
}
