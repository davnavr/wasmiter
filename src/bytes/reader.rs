use crate::bytes::Bytes;

/// Wraps an instance of [`Bytes`] to provide a
/// [`std::io::Read`](https://doc.rust-lang.org/std/io/trait.Read.html) and
/// [`std::io::Seek`](https://doc.rust-lang.org/std/io/trait.Seek.html) implementation.
#[derive(Clone, Copy, Debug)]
pub struct Reader<B: Bytes> {
    offset: u64,
    bytes: B,
}

impl<B: Bytes> Reader<B> {
    /// Creates a new [`Reader`] over the given [`Bytes`], initially positioned at the given `offset`.
    #[inline]
    pub const fn with_offset(offset: u64, bytes: B) -> Self {
        Self { offset, bytes }
    }

    /// Creates a new [`Reader`] over the given [`Bytes`].
    #[inline]
    pub const fn new(bytes: B) -> Self {
        Self::with_offset(0, bytes)
    }

    /// Gets the current offset into the [`Bytes`].
    #[inline]
    pub fn offset(&self) -> u64 {
        self.offset
    }

    /// Sets the current offset into the [`Bytes`].
    #[inline]
    pub fn set_offset(&mut self, offset: u64) {
        self.offset = offset;
    }

    /// Gets the underlying [`Bytes`].
    #[inline]
    pub fn into_bytes(self) -> B {
        self.bytes
    }

    /// Gets a reference to the underlying [`Bytes`].
    #[inline]
    pub fn as_bytes(&self) -> &B {
        &self.bytes
    }
}

impl<B: Bytes> From<B> for Reader<B> {
    #[inline]
    fn from(bytes: B) -> Self {
        Self::new(bytes)
    }
}

impl<B: Bytes> std::io::Read for Reader<B> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        Ok(self.bytes.read(buf, &mut self.offset)?.len())
    }

    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<()> {
        self.bytes
            .read_exact(buf, &mut self.offset)
            .map_err(Into::into)
    }
}

impl<B: Bytes> std::io::Seek for Reader<B> {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        match pos {
            std::io::SeekFrom::Start(offset) => {
                self.offset = offset;
                Ok(offset)
            }
            std::io::SeekFrom::Current(amount) => {
                todo!("from current u64 + i64")
            }
            std::io::SeekFrom::End(amount) => {
                //self.bytes.length_at(0)
                todo!("from end u64 - i64")
            }
        }
    }

    #[inline]
    fn rewind(&mut self) -> std::io::Result<()> {
        self.set_offset(0);
        Ok(())
    }

    #[inline]
    fn stream_position(&mut self) -> std::io::Result<u64> {
        Ok(self.offset())
    }
}
