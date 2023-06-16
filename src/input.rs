//! Traits and type definitions for reading and copying bytes from a source.
//!
//! The [`Input`] trait provides this functionality, and is used by [`wasmiter`](crate) to parse a
//! WebAssembly binary from a source.

mod borrowed;
mod error;
mod hex_dump;
mod input_impls;
mod window;

pub use borrowed::BorrowInput;
pub use error::Error;
pub use hex_dump::{HexDump, HexDumpRow};
pub use window::Window;

/// Result type used when an operation to read [`Input`] fails.
pub type Result<T> = core::result::Result<T, Error>;

#[cold]
#[inline(never)]
pub(crate) fn offset_overflowed(offset: u64) -> Error {
    Error::new(error::ErrorKind::OffsetOverflow, offset, None)
}

#[inline]
pub(crate) fn increment_offset<A: TryInto<u64>>(offset: &mut u64, amount: A) -> Result<()> {
    match amount
        .try_into()
        .ok()
        .and_then(|len| offset.checked_add(len))
    {
        Some(incremented) => {
            *offset = incremented;
            Ok(())
        }
        None => Err(offset_overflowed(*offset)),
    }
}

/// Trait for reading and copying bytes at specific locations from a source.
///
/// This trait is essentially a combination of the [`std::io::Read`] and [`std::io::Seek`] traits.
///
/// [`std::io::Read`]: https://doc.rust-lang.org/std/io/trait.Read.html
/// [`std::io::Seek`]: https://doc.rust-lang.org/std/io/trait.Seek.html
pub trait Input {
    /// Reads bytes starting at the given `offset`, copying them into the `buffer`. Returns the
    /// portion of the `buffer` that was actually copied to.
    ///
    /// An empty slice (`Ok(&[])`) is returned to indicate no bytes were copied into the `buffer`.
    ///
    /// Attempts to read at an `offset` considered "out of bounds" may result in an error or no
    /// bytes being copied. The exact behavior is implementation defined.
    ///
    /// # Errors
    ///
    /// The attempt to read the bytes failed for some reason, such as an I/O error.
    fn read_at<'b>(&self, offset: u64, buffer: &'b mut [u8]) -> Result<&'b mut [u8]>;

    /// Calculates the maximum number of bytes that can at read the given `offset`.
    ///
    /// Attempts to calculate the length at an `offset` considered "out of bounds" may result in an
    /// error or `0` being returned. The exact behavior is implementation defined.
    ///
    /// # Errors
    ///
    /// The length could not be calculated for some reason, such as an I/O error.
    fn length_at(&self, offset: u64) -> Result<u64>;

    /// Reads an exact number of bytes starting at the given `offset`, and copies them into the
    /// `buffer`.
    ///
    /// See the documentation for [`read_at`](Input::read_at) for more information.
    ///
    /// # Errors
    ///
    /// If the `buffer` is not completely filled or the read failed, an error is returned.
    fn read_exact_at(&self, offset: u64, buffer: &mut [u8]) -> Result<()> {
        let buffer_length = buffer.len();
        let copied = self.read_at(offset, buffer)?;

        if copied.len() != buffer_length {
            return Err(Error::new(
                error::ErrorKind::CannotFillBuffer,
                offset,
                copied.len().try_into().ok(),
            ));
        }

        Ok(())
    }

    /// Borrows the current [`Input`] instance, using a returned reference that also implements
    /// [`Input`] instead.
    #[inline]
    fn by_ref(&self) -> &Self {
        self
    }

    /// Reads bytes starting at the given `offset`, copying them into the `buffer`, and then
    /// advances the `offset` by the number of bytes that were read.
    ///
    /// See the documentation for [`read_at`](Input::read_at) for more information.
    ///
    /// # Errors
    ///
    /// Returns an error if the read failed, or if the `offset` would overflow.
    #[inline]
    fn read<'b>(&self, offset: &mut u64, buffer: &'b mut [u8]) -> Result<&'b mut [u8]> {
        let output = self.read_at(*offset, buffer)?;
        increment_offset(offset, output.len())?;
        Ok(output)
    }

    /// Reads an exact number of bytes starting at the given `offset`, copying them into the
    /// `buffer`, and then advances the `offset` by the number of bytes that were read.
    ///
    /// See the documentation for [`read_exact_at`](Input::read_exact_at) for more information.
    ///
    /// # Errors
    ///
    /// Returns an error if the read failed, the `offset` would overflow, or if the `buffer` was
    /// not completely filled.
    #[inline]
    fn read_exact(&self, offset: &mut u64, buffer: &mut [u8]) -> Result<()> {
        self.read_exact_at(*offset, buffer)?;
        increment_offset(offset, buffer.len())
    }
}
