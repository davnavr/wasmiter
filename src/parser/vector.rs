use crate::parser::{input::Input, Parser, Decoder, Result};

use super::ResultExt;

/// Parses a sequence of elements prefixed by a `u32` length.
pub struct Vector<I: Input, P: Parser> {
    count: usize,
    source: Decoder<I>,
    parser: P,
}

impl<I: Input, P: Parser> Vector<I, P> {
    /// Uses the given [`Parser`] to decode a sequence of elements.
    pub fn new(mut source: Decoder<I>, parser: P) -> Result<Self> {
        Ok(Self {
            count: source.leb128_usize().context("vector length")?,
            source,
            parser,
        })
    }

    fn fork(&self) -> Result<Vector<I::Fork, P>> {
        Ok(Vector {
            count: self.count,
            source: self.source.fork()?,
            parser: self.parser.clone(),
        })
    }
}

impl<I: Input> Decoder<I> {
    /// Reads a [`Vector`] of elements.
    ///
    /// See [`Vector::new`] for more information.
    pub fn vector<'a, P: Parser>(&'a mut self, parser: P) -> Result<Vector<&'a mut I, P>> {
        Vector::new(self.by_ref(), parser)
    }
}

impl<I: Input, P: Parser> Iterator for Vector<I, P> {
    type Item = Result<P::Output>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count == 0 {
            return None;
        }

        let mut output = self.parser.parse(&mut self.source);

        self.count -= 1;

        #[cfg(feature = "alloc")]
        if output.is_err() {
            output = output.context(alloc::format!("{} elements remain", self.count));
        }

        Some(output)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.count, Some(self.count))
    }
}

impl<I: Input, P: Parser> ExactSizeIterator for Vector<I, P> {
    #[inline]
    fn len(&self) -> usize {
        self.count
    }
}

impl<I, P> core::fmt::Debug for Vector<I, P>
where
    I: Input,
    P: Parser,
    P::Output: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut list = f.debug_list();
        match self.fork() {
            Ok(forked) => list.entries(forked),
            Err(e) => list.entries(core::iter::once(Result::<()>::Err(e))),
        }
        .finish()
    }
}
