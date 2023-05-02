use crate::parser::input::{Input, Result};

#[cfg(feature = "std")]
use std::io::Read;

/// Allows treating an in-memory buffer as an [`Input`](crate::parser::input::Input).
///
/// This type is a mirror of
/// [`std::io::Cursor<T>`](https://doc.rust-lang.org/std/io/struct.Cursor.html).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[cfg(not(feature = "std"))]
pub struct Cursor<T> {
    buffer: T,
    offset: u64,
}

#[cfg(not(feature = "std"))]
impl<T> Cursor<T> {
    /// Creates a new [`Cursor<T>`] with the specified buffer.
    pub const fn new(buffer: T) -> Self {
        Self { buffer, offset: 0 }
    }

    /// Returns the underlying buffer.
    #[inline]
    pub fn into_inner(self) -> T {
        self.buffer
    }

    /// Gets a reference to the underlying buffer.
    #[inline]
    pub fn get_ref(&self) -> &T {
        &self.buffer
    }
}

#[cfg(not(feature = "std"))]
impl<T: AsRef<[u8]>> Cursor<T> {
    #[inline(always)]
    fn peek_implementation(&mut self, buffer: &mut [u8]) -> Option<usize> {
        let source_start = self
            .buffer
            .as_ref()
            .get(usize::try_from(self.offset).ok()?..)?;
        let length = core::cmp::min(source_start.len(), buffer.len());
        let source_slice = source_start.get(..length)?;
        buffer[..length].copy_from_slice(source_slice);
        Some(length)
    }
}

#[cfg(not(feature = "std"))]
impl<T: AsRef<[u8]>> Input for Cursor<T> {
    #[inline]
    fn seek(&mut self, offset: u64) -> Result<()> {
        self.offset = offset;
        Ok(())
    }

    fn read(&mut self, amount: u64) -> Result<u64> {
        let original_offset = self.offset;
        let target = u64::checked_add(original_offset, amount).unwrap_or(u64::MAX);
        self.offset = target;
        Ok(target - original_offset)
    }

    fn peek(&mut self, buffer: &mut [u8]) -> Result<usize> {
        if let Some(amount) = self.peek_implementation(buffer) {
            Ok(amount)
        } else {
            Ok(0)
        }
    }

    #[inline]
    fn position(&self) -> Result<u64> {
        Ok(self.offset)
    }
}

#[cfg(feature = "std")]
pub use std::io::Cursor;

#[cfg(feature = "std")]
impl<T: AsRef<[u8]>> Input for Cursor<T> {
    #[inline]
    fn seek(&mut self, offset: u64) -> Result<()> {
        Cursor::set_position(self, offset);
        Ok(())
    }

    fn read(&mut self, amount: u64) -> Result<u64> {
        let position = Cursor::position(self);
        let target = u64::checked_add(position, amount).unwrap_or(u64::MAX);
        Input::seek(self, target)?;
        Ok(target - position)
    }

    fn peek(&mut self, buffer: &mut [u8]) -> Result<usize> {
        let position = Cursor::position(self);
        let amount = Read::read(self, buffer)?;
        Cursor::set_position(self, position);
        Ok(amount)
    }

    #[inline]
    fn take(&mut self, buffer: &mut [u8]) -> Result<usize> {
        Ok(Read::read(self, buffer)?)
    }

    #[inline]
    fn take_exact(&mut self, buffer: &mut [u8]) -> Result<()> {
        Ok(Read::read_exact(self, buffer)?)
    }

    #[inline]
    fn position(&self) -> Result<u64> {
        Ok(Cursor::position(self))
    }
}
