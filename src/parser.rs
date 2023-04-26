//! Low-level types and functions for parsing.

mod input;

pub use input::{FileInput, FileReader};

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
pub struct Parser<I: input::Input> {
    input: I,
}

/// Parses the sections of a WebAssembly module.
#[derive(Debug)]
pub struct SectionsParser {}

impl SectionsParser {
    pub(crate) fn from_input<I: input::Input>(input: I) -> Result<Self> {
        todo!()
    }
}
