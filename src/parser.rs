//! Low-level types and functions for parsing.

mod input;

pub use input::{FileInput, FileReader, Input, ToInput};

use std::borrow::Cow;
use std::fmt::Display;
use std::io::Read;

/// Specifies what kind of error occured during parsing.
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum ErrorKind {
    /// An I/O error occured.
    #[error(transparent)]
    IO(std::io::Error),
    /// The input is malformed.
    #[error("input was malformed")]
    InvalidFormat,
}

/// Adds additional information to an [`Error`].
pub struct Context {
    message: Cow<'static, str>,
}

impl From<Cow<'static, str>> for Context {
    #[inline]
    fn from(message: Cow<'static, str>) -> Self {
        Self { message }
    }
}

impl From<&'static str> for Context {
    #[inline]
    fn from(message: &'static str) -> Self {
        Self::from(Cow::Borrowed(message))
    }
}

impl From<String> for Context {
    #[inline]
    fn from(message: String) -> Self {
        Self::from(Cow::Owned(message))
    }
}

/*
impl<'a> From<std::fmt::Arguments<'a>> for Context {
    fn from(arguments: std::fmt::Arguments<'a>) -> Self {
        Self {
            message: if let Some(literal) = arguments.as_str() {
                Cow::Borrowed(literal)
            } else {
                Cow::Owned(arguments.to_string())
            },
        }
    }
}
*/

impl Display for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.message, f)
    }
}

impl std::fmt::Debug for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.message, f)
    }
}

struct ErrorInner {
    kind: ErrorKind,
    context: Vec<Context>,
    backtrace: std::backtrace::Backtrace,
}

/// Describes an error that occured during parsing.
#[repr(transparent)]
pub struct Error {
    inner: Box<ErrorInner>,
}

impl Error {
    fn new(kind: ErrorKind) -> Self {
        Self {
            inner: Box::new(ErrorInner {
                kind,
                context: Vec::new(),
                backtrace: std::backtrace::Backtrace::capture(),
            }),
        }
    }

    pub(crate) fn bad_format() -> Self {
        Self::new(ErrorKind::InvalidFormat)
    }

    /// Gets the kind of error.
    pub fn kind(&self) -> &ErrorKind {
        &self.inner.kind
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

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Error")
            .field("kind", self.kind())
            .field("context", &self.inner.context)
            .field("backtrace", &self.inner.backtrace)
            .finish()
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::new(ErrorKind::IO(error))
    }
}

/// Result type used when parsing input.
pub type Result<T> = std::result::Result<T, Error>;

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
}

impl<'a, R: Read + ToInput<'a>> ToInput<'a> for Parser<R> {
    type In = R::In;

    #[inline]
    fn to_input(&self) -> std::io::Result<Self::In> {
        self.reader.into_input()
    }
}
