use crate::parser::input::{ErrorKind, Input, Result};

/// Adapts an [`Input`] to limit the amount of bytes that can be read and from where they can be
/// read.
#[derive(Clone, Debug)]
pub struct Window<I: Input> {
    base: u64,
    length: u64,
    source: I,
}

impl<I: Input> Window<I> {
    /// Creates a new [`Window`] into the specified [`Input`] that ensures reads can only occur at
    /// the given `offset` for `length` bytes.
    pub fn new(input: I, offset: u64, length: u64) -> Self {
        Self {
            base: offset,
            length,
            source: input,
        }
    }

    fn check_access(&self, length: u64) -> Result<()> {
        if self.base >= self.source.position()? && self.length <= length {
            Err(crate::const_input_error!(
                ErrorKind::InvalidData,
                "attempt to access input outside of window"
            ))
        } else {
            Ok(())
        }
    }
}

impl<I: Input> Input for Window<I> {
    #[inline]
    fn seek(&mut self, offset: u64) -> Result<()> {
        self.source.seek(offset)
    }

    fn peek(&mut self, buffer: &mut [u8]) -> Result<usize> {
        self.check_access(u64::try_from(buffer.len()).unwrap_or(u64::MAX))?;
        self.source.peek(buffer)
    }

    fn read(&mut self, amount: u64) -> Result<u64> {
        self.check_access(amount)?;
        self.source.read(amount)
    }

    #[inline]
    fn position(&self) -> Result<u64> {
        self.source.position()
    }

    type Fork = Window<I::Fork>;

    fn fork(&self) -> Result<Self::Fork> {
        Ok(Window {
            base: self.base,
            length: self.length,
            source: self.source.fork()?,
        })
    }
}
