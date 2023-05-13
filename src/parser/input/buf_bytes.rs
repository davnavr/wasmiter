use crate::parser::input::{Bytes, Result};

struct Cache<U> {
    /// Buffered bytes, where `buffer[0]` refers to the byte at `cached_range.start`.
    buffer: U,
    offsets: core::ops::Range<u64>,
}

/// Provides an in-memory buffer to cache results from [`Bytes`].
///
/// Certain [`Bytes`] implementations such as
/// [`SharedBytes<I>`](crate::parser::input::SharedBytes) can have poor performance when used
/// without buffering, such as in the case of [`std::io::File`] where many syscalls can occur even
/// if reads occur that would
/// [normally not benefit from buffering](https://doc.rust-lang.org/std/io/struct.BufWriter.html).
pub struct BufBytes<U: AsMut<[u8]>, B: Bytes> {
    cache: core::cell::RefCell<Cache<U>>,
    inner: B,
}

impl<U: AsMut<[u8]>, B: Bytes> BufBytes<U, B> {
    /// Creates a new [`BufBytes`] with the given buffer.
    pub fn with_buffer(buffer: U, inner: B) -> Self {
        Self {
            cache: core::cell::RefCell::new(Cache {
                buffer,
                offsets: 0..0,
            }),
            inner,
        }
    }

    /// Gets a reference to the [`Bytes`].
    #[inline]
    pub fn get_ref(&self) -> &B {
        &self.inner
    }

    /// Gets the underlying [`Bytes`].
    #[inline]
    pub fn into_inner(self) -> B {
        self.inner
    }
}

#[cfg(feature = "alloc")]
impl<B: Bytes> BufBytes<alloc::vec::Vec<u8>, B> {
    /// Creates a new [`BufBytes`] with a buffer of the given `capacity`, in bytes.
    #[inline]
    pub fn with_capacity(capacity: usize, inner: B) -> Self {
        Self::with_buffer(alloc::vec::Vec::with_capacity(capacity), inner)
    }

    const DEFAULT_CAPACITY: usize = 1024 * 8;

    /// Creates a new [`BufBytes`] with a buffer with the default capacity, which is currently 8 KiB.
    #[inline]
    pub fn new(inner: B) -> Self {
        Self::with_capacity(Self::DEFAULT_CAPACITY, inner)
    }
}

impl<U: AsMut<[u8]>, B: Bytes> Bytes for BufBytes<U, B> {
    fn read_at<'b>(&self, offset: u64, buffer: &'b mut [u8]) -> Result<&'b mut [u8]> {
        let output_length = u64::try_from(buffer.len()).unwrap_or(u64::MAX);
        let mut copied = 0usize;
        let mut cache = self.cache.borrow_mut();

        // Copy bytes from buffer into output
        if offset < cache.offsets.end && offset + output_length > cache.offsets.start {
            copied +=
                core::cmp::min(output_length, cache.offsets.end - cache.offsets.start) as usize;

            let start_index = (cache.offsets.start - offset) as usize;
            buffer[..copied].copy_from_slice(&cache.buffer.as_mut()[start_index..][..copied]);
        }

        // If buffer only contained some of the bytes, then the rest need to be read from inner
        if copied != buffer.len() {
            // TODO: If backtracking (cache.offsets.start < offset), then parts of cache.buffer can be/need to be moved around

            match self.read_at(offset, cache.buffer.as_mut()) {
                Ok(cache_slice) => {
                    cache.offsets = offset..offset + (cache_slice.len() as u64);
                }
                Err(e) => {
                    // buffer may be partially filled if error occurs, so it is invalidated
                    cache.offsets = 0..0;
                    return Err(e);
                }
            }

            // Copy newly read bytes
            let remaining = core::cmp::min(buffer.len() - copied, cache.buffer.as_mut().len());
            buffer[copied..][..remaining].copy_from_slice(&cache.buffer.as_mut()[..remaining]);
            copied += remaining;
        }

        Ok(&mut buffer[..copied])
    }

    #[inline]
    fn length_at(&self, offset: u64) -> Result<u64> {
        // TODO: maybe call length_at in constructor and cache it?
        self.inner.length_at(offset)
    }
}

impl<U: AsMut<[u8]>, B: Bytes + std::fmt::Debug> std::fmt::Debug for BufBytes<U, B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("BufBytes")
            .field(
                "offsets",
                &self
                    .cache
                    .try_borrow()
                    .ok()
                    .map(|cache| core::cell::Ref::map(cache, |cache| &cache.offsets)),
            )
            .field("inner", &self.inner)
            .finish()
    }
}
