//! Low-level types and functions for parsing.

mod input;

pub use input::{FileInput, FileReader, Input};

/// Specifies what kind of error occured during parsing.
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum ErrorKind {
    /// An [`std::io::Error`].
    #[error(transparent)]
    IO(std::io::Error),
}

struct ErrorInner {
    kind: ErrorKind,
}

/// Describes an error that occured during parsing.
#[repr(transparent)]
pub struct Error {
    inner: Box<ErrorInner>,
}

impl Error {
    fn new(kind: ErrorKind) -> Self {
        Self {
            inner: Box::new(ErrorInner { kind }),
        }
    }

    /// Gets the kind of error.
    pub fn kind(&self) -> &ErrorKind {
        &self.inner.kind
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Error").field("kind", self.kind()).finish()
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::new(ErrorKind::IO(error))
    }
}

/// Result type used when parsing input.
pub type Result<T> = std::result::Result<T, Error>;

/// Parses a stream of bytes.
#[derive(Debug)]
pub struct Parser<I: Input> {
    input: I,
}

impl<I: Input> Parser<I> {
    /// Creates a new parser over the specified [`Input`].
    pub fn new(input: I) -> Self {
        Self { input }
    }
}
