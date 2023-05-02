//! Traits and type definitions for reading bytes from a source.

mod cursor;
mod error;

pub use cursor::Cursor;
pub use error::{Error, ErrorKind};

macro_rules! const_input_error {
    ($kind:expr, $message:literal) => {{
        const ERROR: &error::ConstantError = &error::ConstantError::new($kind, $message);
        Error::from_const(ERROR)
    }};
}

/// Result type used when an operation with an [`Input`] fails.
///
/// This type is meant to be a mirror of
/// [`std::io::Result`](https://doc.rust-lang.org/std/io/type.Result.html).
pub type Result<T> = core::result::Result<T, Error>;

/// Trait for reading bytes at specific locations from a source.
///
/// This trait serves as a combination of the
/// [`std::io::Read`](https://doc.rust-lang.org/std/io/trait.Read.html) and
/// [`std::io::Seek`](https://doc.rust-lang.org/std/io/trait.Seek.html) traits, but with a
/// few differences.
pub trait Input {
    /// Moves the reader to a location specified by a byte `offset` from the start of the source.
    fn seek(&mut self, offset: u64) -> Result<()>;

    /// Reads bytes starting at the current [`position`](Input::position) without advancing the
    /// reader. Returns the number of bytes copied from the source to the `buffer`.
    fn peek(&mut self, buffer: &mut [u8]) -> Result<usize>;

    // TODO: struct PeekBuffers<'a> { read: &'b mut [u8], unfilled: usize } use split_at_mut
    //Returns the portion of the `buffer` filled with the bytes read from the source, and the remaining portion of the `buffer`.
    //fn peek_bytes<'b>(&mut self, buffer: &'b mut [u8]) -> Result<PeekBuffers<'b>>

    /// Advances the reader by the given byte `amount`, returning the number of bytes that were
    /// skipped.
    ///
    /// This is equivalent to calling [`seek`](Input::seek) with the current
    /// [`position`](Input::position) plus `amount`.
    fn read(&mut self, amount: u64) -> Result<u64>;

    /// Returns the current position of the reader, as a byte offset from the start of the source.
    fn position(&self) -> Result<u64>; // u64?

    //fn fork(&self) -> Result<Self::Fork>;

    /// Reads bytes to fill the `buffer`, advancing the reader by the number of bytes that were
    /// read. Returns the number of bytes copied to the `buffer`.
    ///
    /// This is equivalent to calling [`peek`](Input::peek) followed by call to
    /// [`read`](Input::read).
    ///
    /// This method is the equivalent of
    /// [`std::io::Read::read`](https://doc.rust-lang.org/std/io/trait.Read.html#tymethod.read).
    fn take(&mut self, buffer: &mut [u8]) -> Result<usize> {
        #[cfg(feature = "std")]
        let offset_overflowed = |e| Error::from(std::io::Error::new(ErrorKind::InvalidData, e));

        #[cfg(not(feature = "std"))]
        let offset_overflowed = |_| {
            const_input_error!(
                ErrorKind::InvalidData,
                "reader position overflowed while filling buffer"
            )
        };

        let amount = self.peek(buffer)?;
        self.read(u64::try_from(amount).map_err(offset_overflowed)?)?;
        Ok(amount)
    }

    /// A variant of [`Input::take` that attempts to completely fill the `buffer`, returning an
    /// error otherwise.
    ///
    /// This method is the equivalent of
    /// [`std::io::Read::read_exact`](https://doc.rust-lang.org/std/io/trait.Read.html#method.read_exact).
    fn take_exact(&mut self, buffer: &mut [u8]) -> Result<()> {
        let amount = self.take(buffer)?;

        if amount != buffer.len() {
            return Err(const_input_error!(
                ErrorKind::UnexpectedEof,
                "buffer could not be completely filled"
            ));
        }

        Ok(())
    }

    // TODO: functions to help with caches/buffers, no-op default impl
}
