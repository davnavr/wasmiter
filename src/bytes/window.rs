use crate::bytes::{self, Bytes, Result};

/// Adapts a [`Bytes`] implementation to limit the amount of bytes that can be read to a specific
/// range.
///
/// To instead treat the first readab;e byte within the region of the [`Window`] as starting at
/// offset `0`, see the [`BytesSlice`](bytes::BytesSlice) struct.
#[derive(Clone, Copy)]
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

    /// Gets the offset at which the [`Window`] content begins.
    #[inline]
    pub fn base(&self) -> u64 {
        self.base
    }

    /// Gets length of the [`Window`].
    #[inline]
    pub fn length(&self) -> u64 {
        self.length
    }

    /// Gets a reference to the inner [`Bytes`].
    #[inline]
    pub fn as_inner(&self) -> &B {
        &self.inner
    }

    #[inline]
    pub(super) fn into_inner(self) -> B {
        self.inner
    }

    pub(crate) fn borrowed(&self) -> Window<&B> {
        Window {
            base: self.base,
            length: self.length,
            inner: &self.inner,
        }
    }
}

impl<B: Bytes + Clone> Window<&B> {
    pub(crate) fn cloned(&self) -> Window<B> {
        Window {
            base: self.base,
            length: self.length,
            inner: self.inner.clone(),
        }
    }
}

impl<B: Bytes> Window<Window<B>> {
    /// Flattens a [`Window`], with the resulting [`Bytes`] being constrained to the portion of the
    /// inner [`Window`] that is accessible.
    pub fn flatten(self) -> Window<B> {
        if self.inner.base > self.base + self.length
            || self.base > self.inner.base + self.inner.length
        {
            // No overlap
            Window {
                base: self.inner.base,
                length: 0,
                inner: self.inner.inner,
            }
        } else if self.inner.base >= self.base {
            Window {
                base: self.inner.base,
                length: core::cmp::min(
                    self.inner.length,
                    self.length - (self.inner.base - self.base),
                ),
                inner: self.inner.inner,
            }
        } else {
            // self.inner.base < self.base
            Window {
                base: self.base,
                length: core::cmp::min(
                    self.length,
                    self.inner.length - (self.base - self.inner.base),
                ),
                inner: self.inner.inner,
            }
        }
    }
}

impl<B: Bytes> Bytes for Window<B> {
    fn read_at<'b>(&self, offset: u64, buffer: &'b mut [u8]) -> Result<&'b mut [u8]> {
        match u64::try_from(buffer.len()) {
            Ok(buffer_length) if offset >= self.base => {
                let actual_buffer = if buffer_length <= self.length {
                    buffer
                } else {
                    &mut buffer[..self.length as usize]
                };

                self.inner.read_at(offset, actual_buffer)
            }
            _ => Ok(Default::default()),
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

impl<B: Bytes> core::fmt::Debug for Window<B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Window")
            .field("base", &self.base)
            .field("length", &self.length)
            .field(
                "content",
                &bytes::DebugBytes::from(bytes::BytesSlice::from_window(self.borrowed())),
            )
            .finish()
    }
}
