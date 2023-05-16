use crate::bytes::{Bytes, Result};

/// Slices a [`Bytes`] implementation.
///
/// To instead limit reads from the original [`Bytes`] rather than hiding the regions outside of
/// the slice, see the [`Window`](crate::bytes::Window) struct.
#[derive(Clone, Copy)]
pub struct BytesSlice<B: Bytes> {
    start: u64,
    length: u64,
    inner: B,
}

impl<B: Bytes> BytesSlice<B> {
    /// Creates a new slice into the specified [`Bytes`], starting at the given `offset`
    /// for `length` bytes.
    pub fn new(inner: B, offset: u64, length: u64) -> Self {
        Self {
            start: offset,
            length,
            inner,
        }
    }

    /// Gets length of the slice.
    #[inline]
    pub fn length(&self) -> u64 {
        self.length
    }
}

impl<B: Bytes> Bytes for BytesSlice<B> {
    fn read_at<'b>(&self, offset: u64, buffer: &'b mut [u8]) -> Result<&'b mut [u8]> {
        if let Some(actual_offset) = offset.checked_add(self.start) {
            let requested_length = buffer.len();
            self.inner.read_at(
                actual_offset,
                &mut buffer[0..core::cmp::min(
                    requested_length,
                    usize::try_from(self.length).unwrap_or(usize::MAX),
                )],
            )
        } else {
            Ok(&mut [])
        }
    }

    fn length_at(&self, offset: u64) -> Result<u64> {
        Ok(
            if let Some(actual_offset) = offset.checked_add(self.start) {
                core::cmp::min(self.inner.length_at(actual_offset)?, self.length)
            } else {
                0
            },
        )
    }
}

impl<B: Bytes> core::fmt::Debug for BytesSlice<B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("BytesSlice")
            .field("length", &self.length)
            .field("content", &crate::bytes::BytesDebug::from(self))
            .finish()
    }
}
