use crate::parser::input::{Bytes, Error, ErrorKind, Result};

#[cold]
#[inline(never)]
fn offset_overflowed() -> Error {
    crate::const_input_error!(ErrorKind::UnexpectedEof, "reader offset overflowed")
}

/// A cursor used to read from [`Bytes`].
///
/// This is essentially an equivalent to
/// [`std::io::Cursor`](https://doc.rust-lang.org/std/io/struct.Cursor.html).
///
/// If the `std` feature is enabled, then
/// [`std::io::Read`](https://doc.rust-lang.org/std/io/trait.Read.html) and
/// [`std::io::Seek`](https://doc.rust-lang.org/std/io/trait.Seek.html) implementations are also
/// provided.
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

    #[inline]
    fn advance(&mut self, amount: usize) -> Result<()> {
        let new_offset = u64::try_from(amount)
            .ok()
            .and_then(|amt| self.offset.checked_add(amt));
        self.offset = new_offset.unwrap_or(u64::MAX);
        if new_offset.is_some() {
            Ok(())
        } else {
            Err(offset_overflowed())
        }
    }

    /// Reads bytes and copies them into a `buffer`, advancing the cursor.
    pub fn read<'b>(&mut self, buffer: &'b mut [u8]) -> Result<&'b mut [u8]> {
        let result = self.bytes.read_at(self.offset, buffer);
        if let Ok(slice) = &result {
            self.advance(slice.len())?;
        }
        result
    }

    /// Reads an exact number of bytes and copies them into a `buffer`, advancing the cursor.
    pub fn read_exact(&mut self, buffer: &mut [u8]) -> Result<()> {
        self.bytes.read_at_exact(self.offset, buffer)?;
        self.advance(buffer.len())
    }
}

impl<B: Bytes> From<B> for Reader<B> {
    #[inline]
    fn from(bytes: B) -> Self {
        Self::new(bytes)
    }
}

#[cfg(feature = "std")]
impl<B: Bytes> std::io::Read for Reader<B> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let slice = self.read(buf)?;
        Ok(slice.len())
    }

    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<()> {
        self.read_exact(buf).map_err(Into::into)
    }
}

#[cfg(feature = "std")]
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
