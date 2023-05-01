use crate::parser::input::{Input, Result, ErrorKind};

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
    /// Returns the underlying buffer.
    #[inline]
    pub fn into_inner(self) -> T {
        self.buffer
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

    fn peek<'b>(&mut self, buffer: &'b mut [u8]) -> Result<(&'b [u8], &'b [u8])> {
        let position = Cursor::position(self);
        let amount = Read::read(self, buffer)?;
        Cursor::set_position(self, position);
        Ok(buffer.split_at(amount))
    }

    #[inline]
    fn position(&self) -> Result<u64> {
        Ok(Cursor::position(self))
    }
}
