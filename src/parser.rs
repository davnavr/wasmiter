//! Low-level types and functions for parsing.

mod error;

pub mod input;

pub use error::{Context, Error, ErrorKind};

/// Result type used when parsing input.
pub type Result<T> = core::result::Result<T, Error>;

/*
impl Error {
    pub(crate) fn bad_format() -> Self {
        Self::new(ErrorKind::InvalidFormat)
    }

    #[inline]
    pub(crate) fn context<C: Into<Context>>(&mut self, context: C) {
        self.inner.context.push(context.into())
    }

    #[inline]
    pub(crate) fn with_context<C: Into<Context>>(mut self, context: C) -> Self {
        self.context(context);
        self
    }
}

mod sealed {
    pub trait Sealed {}

    impl<T, E: Into<super::Error>> Sealed for std::result::Result<T, E> {}
}

/// Provides helper methods to add additional context to [`Error`]s.
///
/// This trait is sealed.
pub trait ResultExt<T>: sealed::Sealed {
    /// Attaches the given [`Context`] to the [`Result<T>`], if it is an error.
    fn context<C: Into<Context>>(self, context: C) -> Result<T>
    where
        Self: Sized,
    {
        self.with_context(|| context)
    }

    /// If the given [`Result<T>`] is an error, evaluates the closure to attach [`Context`].
    fn with_context<C: Into<Context>, F: FnOnce() -> C>(self, f: F) -> Result<T>;
}

impl<T, E: Into<Error>> ResultExt<T> for std::result::Result<T, E> {
    fn with_context<C: Into<Context>, F: FnOnce() -> C>(self, f: F) -> Result<T> {
        match self {
            Ok(value) => Ok(value),
            Err(e) => {
                let mut err = e.into();
                err.context(f());
                Err(err)
            }
        }
    }
}

trait IntegerEncoding: From<u8> + Copy + Default + std::ops::BitOr + std::ops::ShlAssign {
    /// A buffer type to contain the maximum number of bytes that a value is allowed to be encoded
    /// in.
    ///
    /// According to the
    /// [WebAssembly specification](https://webassembly.github.io/spec/core/binary/values.html#integers),
    /// this should have a length equal to `ceil(BITS / 7)`.
    type Buffer: AsRef<[u8]> + Default;
}

impl IntegerEncoding for u32 {
    type Buffer = [u8; 5];
}

impl IntegerEncoding for u64 {
    type Buffer = [u8; 10];
}

/// Parses a stream of bytes.
#[derive(Debug)]
pub struct Parser<R: Read> {
    reader: R,
}

impl<R: Read> Parser<R> {
    /// Creates a new parser with the specified reader.
    pub fn new(reader: R) -> Self {
        Self { reader }
    }

    /// Fills the given `buffer` with bytes from the reader, returning an error if the buffer could
    /// not be completely filled.
    pub fn bytes_exact(&mut self, buffer: &mut [u8]) -> Result<()> {
        // TODO: Use stream_position to get context for location Option<u64>
        let count = self
            .reader
            .read(buffer)
            .with_context(|| format!("attempt to read {} bytes", buffer.len()))?;

        if count != buffer.len() {
            return Err(Error::bad_format()
                .with_context(format!("expected {} bytes but got {count}", buffer.len())));
        }

        Ok(())
    }

    fn leb128_unsigned<I: IntegerEncoding>(&mut self) -> Result<I> {
        let mut buffer = I::Buffer::default();
        let mut value = I::default();
        todo!()
    }
}
*/
