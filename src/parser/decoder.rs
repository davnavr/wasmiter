use crate::bytes::{self, Bytes, Reader};
use crate::parser::{Error, Result, ResultExt};

/// Wraps a [`Reader`] to parse structures from a source.
#[derive(Clone, Copy, Debug)]
pub struct Decoder<B: Bytes> {
    reader: Reader<B>,
}

impl<B: Bytes> Decoder<B> {
    /// Creates a new [`Decoder`] with the specified [`Reader`].
    pub fn new(reader: Reader<B>) -> Self {
        Self { reader }
    }

    /// Creates a new [`Decoder`] to parse structures from the specified [`Bytes`].
    pub fn from_bytes(bytes: B) -> Self {
        Self::new(Reader::new(bytes))
    }

    #[inline]
    pub(crate) fn position(&self) -> Result<u64> {
        self.input.position().map_err(Error::from)
    }

    

    

    

    pub(crate) fn bytes(&mut self, buffer: &mut [u8]) -> Result<usize> {
        self.input
            .take(buffer)
            .map_err(|e| parser_bad_input!(e, "could not read {} bytes", buffer.len()))
    }

    pub(crate) fn bytes_exact(&mut self, buffer: &mut [u8]) -> Result<()> {
        self.input
            .take_exact(buffer)
            .map_err(|e| parser_bad_input!(e, "expected {} bytes", buffer.len()))
    }

    pub(crate) fn one_byte(&mut self) -> Result<Option<u8>> {
        let mut value = 0u8;
        Ok(if self.bytes(core::slice::from_mut(&mut value))? == 0 {
            None
        } else {
            Some(value)
        })
    }

    pub(crate) fn one_byte_exact(&mut self) -> Result<u8> {
        let mut value = 0u8;
        self.bytes_exact(core::slice::from_mut(&mut value))?;
        Ok(value)
    }

    pub(crate) fn skip_exact(&mut self, amount: u64) -> Result<()> {
        let actual = self
            .input
            .read(amount)
            .map_err(|e| parser_bad_input!(e, "could not read {amount} bytes"))?;

        if amount != actual {
            return Err(parser_bad_format!(
                "attempt to read {amount} bytes, but read {actual} before reaching end of input"
            ));
        }

        Ok(())
    }

    /// Parses an UTF-8 string
    /// [name](https://webassembly.github.io/spec/core/binary/values.html#names).
    pub fn name<'b, U: crate::allocator::Buffer>(
        &mut self,
        buffer: &'b mut U,
    ) -> Result<&'b mut str> {
        let length = self.leb128_usize().context("string length")?;
        buffer.clear();
        buffer.grow(length);

        self.bytes_exact(buffer.as_mut())
            .context("string contents")?;

        core::str::from_utf8_mut(buffer.as_mut()).map_err(|e| crate::parser_bad_format!("{e}"))
    }
}
