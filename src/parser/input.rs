//! Traits and type definitions for reading bytes from a source.

mod cursor;
mod error;
mod window;

pub use cursor::Cursor;
pub use error::{Error, ErrorKind};
pub use window::Window;

#[doc(hidden)]
#[macro_export]
macro_rules! const_input_error {
    ($kind:expr, $message:literal) => {{
        const ERROR: &$crate::parser::input::error::ConstantError =
            &$crate::parser::input::error::ConstantError::new($kind, $message);
        crate::parser::input::Error::from_const(ERROR)
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
    /// An [`Input`] implementation used to read bytes from an existing [`Input`].
    ///
    /// See [`Input::fork`] for more information.
    type Fork: Input<Fork = Self::Fork>;

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

    /// Returns a new [`Input`] used to read bytes starting at the current
    /// [`position`](Input::position).
    fn fork(&self) -> Result<Self::Fork>;

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

    /// Uses the [`Input`] as a mutable reference.
    #[inline]
    fn by_ref(&mut self) -> &mut Self {
        self
    }
}

impl<I: Input> Input for &mut I {
    #[inline]
    fn seek(&mut self, offset: u64) -> Result<()> {
        I::seek(self, offset)
    }

    #[inline]
    fn peek(&mut self, buffer: &mut [u8]) -> Result<usize> {
        I::peek(self, buffer)
    }

    #[inline]
    fn read(&mut self, amount: u64) -> Result<u64> {
        I::read(self, amount)
    }

    #[inline]
    fn position(&self) -> Result<u64> {
        I::position(self)
    }

    type Fork = I::Fork;

    #[inline]
    fn fork(&self) -> Result<Self::Fork> {
        I::fork(&self)
    }

    #[inline]
    fn take(&mut self, buffer: &mut [u8]) -> Result<usize> {
        I::take(self, buffer)
    }

    #[inline]
    fn take_exact(&mut self, buffer: &mut [u8]) -> Result<()> {
        I::take_exact(self, buffer)
    }
}

/// Trait for conversion into an [`Input`].
pub trait IntoInput {
    /// The [`Input`] implementation.
    type In: Input;

    /// Converts a value into an [`Input`].
    fn into_input(self) -> Self::In;
}

impl<I: Input> IntoInput for I {
    type In = I;

    fn into_input(self) -> I {
        self
    }
}

impl<'a> IntoInput for &'a [u8] {
    type In = Cursor<Self>;

    fn into_input(self) -> Self::In {
        Cursor::new(self)
    }
}

impl<'a, const N: usize> IntoInput for &'a [u8; N] {
    type In = Cursor<Self>;

    fn into_input(self) -> Self::In {
        Cursor::new(self)
    }
}

impl<const N: usize> IntoInput for [u8; N] {
    type In = Cursor<Self>;

    fn into_input(self) -> Self::In {
        Cursor::new(self)
    }
}
