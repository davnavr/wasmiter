use crate::bytes::{Bytes, Result};

/// Adapts a [`Bytes`] implementation to limit the amount of bytes that can be read to a specific
/// range.
#[derive(Clone, Copy, Debug)]
pub struct Window<B: Bytes> {
    base: u64,
    length: u64,
    inner: B,
}

impl<B: Bytes> Window<B> {
    /// Creates a new [`Window`] into the specified [`Bytes`] that ensures reads can only occur at
    /// the given `offset` for `length` bytes.
    pub fn new(inner: B, offset: u64, length: u64) -> Self {
        Self {
            base: offset,
            length,
            inner,
        }
    }
}

impl<B: Bytes> Bytes for Window<B> {
    fn read_at<'b>(&self, offset: u64, buffer: &'b mut [u8]) -> Result<&'b mut [u8]> {
        match u64::try_from(buffer) {
            Ok(buffer_length) if offset >= self.base => {
                let actual_buffer = if buffer_length <= self.length {
                    buffer
                } else {
                    &mut buffer[..self.length as usize]
                };

                self.inner.read_at(offset, actual_buffer)
            }
            _ => Ok(&[]),
        }
    }

    fn length_at(&self, offset: u64) -> Result<u64> {
        Ok(if offset >= self.base && offset < self.base + self.length {
            self.length - offset
        } else {
            0
        })
    }
}
