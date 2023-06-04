use core::fmt::{Debug, Display, Formatter};

#[cfg(feature = "std")]
pub use std::io::ErrorKind;

/// Specifies a category for an [`Error`].
///
/// This type is meant to be a subset of
/// [`std::io::ErrorKind`](https://doc.rust-lang.org/std/io/enum.ErrorKind.html),
/// allowing its usage in a `#![no_std]` context.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg(not(feature = "std"))]
pub enum ErrorKind {
    /// Some parameter was incorrect.
    InvalidInput,
    /// Invalid data was read or encountered.
    InvalidData,
    /// The end of the input was unexpectedly encountered.
    UnexpectedEof,
    /// Some unspecified error occured.
    Other,
}

#[derive(Clone, Copy)]
pub(super) struct ConstantError {
    kind: ErrorKind,
    message: &'static str,
}

impl ConstantError {
    pub(super) const fn new(kind: ErrorKind, message: &'static str) -> Self {
        Self { kind, message }
    }
}

/// Error type used when an operation with [`Bytes`](crate::bytes::Bytes) fails.
///
/// This type is meant to be a mirror of
/// [`std::io::Error`](https://doc.rust-lang.org/std/io/struct.Error.html).
#[repr(transparent)]
pub struct Error {
    #[cfg(feature = "std")]
    inner: std::io::Error,
    #[cfg(not(any(feature = "std", feature = "alloc")))]
    inner: &'static ConstantError,
    #[cfg(all(not(feature = "std"), feature = "alloc"))]
    inner: alloc::boxed::Box<ConstantError>,
}

impl Error {
    pub(super) fn from_const(error: &'static ConstantError) -> Self {
        #[cfg(feature = "std")]
        return Self {
            inner: std::io::Error::new(error.kind, error.message),
        };

        #[cfg(not(feature = "std"))]
        {
            #[cfg(feature = "alloc")]
            return Self {
                inner: alloc::boxed::Box::new(*error),
            };

            #[cfg(not(feature = "alloc"))]
            return Self { inner: error };
        }
    }

    /// Gets the category that this error belongs to.
    #[inline]
    pub fn kind(&self) -> ErrorKind {
        #[cfg(feature = "std")]
        return self.inner.kind();

        #[cfg(not(feature = "std"))]
        return self.inner.kind;
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! const_input_error {
    ($kind:expr, $message:literal) => {{
        const ERROR: &$crate::bytes::error::ConstantError =
            &$crate::bytes::error::ConstantError::new($kind, $message);

        $crate::bytes::Error::from_const(ERROR)
    }};
}

#[cfg(feature = "std")]
impl From<std::io::Error> for Error {
    #[inline]
    fn from(error: std::io::Error) -> Self {
        Self { inner: error }
    }
}

#[cfg(feature = "std")]
impl From<Error> for std::io::Error {
    #[inline]
    fn from(error: Error) -> Self {
        error.inner
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut s = f.debug_struct("Error");

        #[cfg(feature = "std")]
        s.field("inner", &self.inner);

        #[cfg(not(feature = "std"))]
        {
            s.field("kind", &self.inner.kind);
            s.field("message", &self.inner.message);
        }

        s.finish()
    }
}

impl Display for Error {
    #[cfg(feature = "std")]
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&self.inner, f)
    }

    #[cfg(not(feature = "std"))]
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&self.inner.message, f)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.inner)
    }
}
