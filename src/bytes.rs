//! Traits and type definitions for reading bytes from a source.

mod buf_bytes;
mod error;
mod reader;
mod window;

#[cfg(feature = "std")]
mod shared_bytes;

pub use buf_bytes::BufBytes;
pub use error::{Error, ErrorKind};
pub use reader::Reader;
pub use shared_bytes::{SharedBytes, SharedInput};
pub use window::Window;

/// Result type used when an operation with an [`Input`] fails.
///
/// This type is meant to be a mirror of
/// [`std::io::Result`](https://doc.rust-lang.org/std/io/type.Result.html).
pub type Result<T> = core::result::Result<T, Error>;

/// Trait for reading bytes at specific locations from a source.
///
/// This trait is essentially a version of the
/// [`std::io::Seek`](https://doc.rust-lang.org/std/io/trait.Seek.html) traits, but with methods
/// modified to accept buffers to write bytes into.
pub trait Bytes {
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

    /// Reads the bytes starting at the given `offset`, and copies them into the `buffer`.
    ///
    /// See the documentation for [`read_at`](Bytes::read_at) for more information.
    ///
    /// # Errors
    ///
    /// If the `buffer` is not completely filled, an error is returned.
    fn read_at_exact(&self, offset: u64, buffer: &mut [u8]) -> Result<()> {
        let buffer_length = buffer.len();
        let copied = self.read_at(offset, buffer)?;

        if copied.len() != buffer_length {
            return Err(crate::const_input_error!(
                ErrorKind::UnexpectedEof,
                "buffer could not be completely filled"
            ));
        }

        Ok(())
    }

    /// Borrows the current [`Bytes`] instance, using a returned reference that also implements
    /// [`Bytes`] instead.
    #[inline]
    fn by_ref(&self) -> &Self {
        self
    }
}

impl<B: Bytes> Bytes for &B {
    #[inline]
    fn read_at<'b>(&self, offset: u64, buffer: &'b mut [u8]) -> Result<&'b mut [u8]> {
        B::read_at(self, offset, buffer)
    }

    #[inline]
    fn length_at(&self, offset: u64) -> Result<u64> {
        B::length_at(self, offset)
    }

    #[inline]
    fn read_at_exact(&self, offset: u64, buffer: &mut [u8]) -> Result<()> {
        B::read_at_exact(self, offset, buffer)
    }
}

impl Bytes for &[u8] {
    fn read_at<'b>(&self, offset: u64, buffer: &'b mut [u8]) -> Result<&'b mut [u8]> {
        let source = usize::try_from(offset)
            .ok()
            .and_then(|start| self.get(start..))
            .unwrap_or_default();

        let copy_amount = core::cmp::min(source.len(), buffer.len());
        buffer.copy_from_slice(&source[..copy_amount]);
        Ok(&mut buffer[..copy_amount])
    }

    fn length_at(&self, offset: u64) -> Result<u64> {
        Ok(if let Ok(start) = usize::try_from(offset) {
            u64::try_from(self.len() - start).unwrap_or(u64::MAX)
        } else {
            // OOB
            0
        })
    }
}
