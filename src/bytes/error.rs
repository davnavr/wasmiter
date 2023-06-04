use core::fmt::{Debug, Display, Formatter};

cfg_if::cfg_if! {
    if #[cfg(feature = "std")] {
        pub use std::io::ErrorKind;
    } else {
        /// Specifies a category for an [`Error`].
        ///
        /// This type is meant to be a subset of [`std::io::ErrorKind`], allowing its usage in a
        /// `#![no_std]` context.
        ///
        /// [`std::io::ErrorKind`]: https://doc.rust-lang.org/std/io/enum.ErrorKind.html
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
    }
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

cfg_if::cfg_if! {
    if #[cfg(feature = "std")] {
        type ErrorInner = std::io::Error;
    } else if #[cfg(feature = "alloc")] {
        type ErrorInner = alloc::boxed::Box<ConstantError>;
    } else {
        type ErrorInner = &'static ConstantError;
    }
}

/// Error type used when an operation with [`Bytes`](crate::bytes::Bytes) fails.
///
/// This type is meant to be a mirror of
/// [`std::io::Error`](https://doc.rust-lang.org/std/io/struct.Error.html).
#[repr(transparent)]
pub struct Error {
    inner: ErrorInner,
}

impl Error {
    pub(super) fn from_const(error: &'static ConstantError) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(feature = "std")] {
                let inner = std::io::Error::new(error.kind, error.message);
            } else if #[cfg(feature = "alloc")] {
                let inner = alloc::boxed::Box::new(*error);
            } else {
                let inner = error;
            }
        };

        Self { inner }
    }

    /// Gets the category that this error belongs to.
    #[inline]
    pub fn kind(&self) -> ErrorKind {
        cfg_if::cfg_if! {
            if #[cfg(feature = "std")] {
                self.inner.kind()
            } else {
                self.inner.kind
            }
        }
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

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        cfg_if::cfg_if! {
            if #[cfg(feature = "std")] {
                Debug::fmt(&self.inner, f)
            } else {
                f.debug_struct("Error")
                    .field("kind", &self.inner.kind)
                    .field("message", &self.inner.message)
                    .finish()
            }
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        cfg_if::cfg_if! {
            if #[cfg(feature = "std")] {
                Display::fmt(&self.inner, f)
            } else {
                Display::fmt(&self.inner.message, f)
            }
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "std")] {
        impl From<std::io::Error> for Error {
            #[inline]
            fn from(error: std::io::Error) -> Self {
                Self { inner: error }
            }
        }

        impl From<Error> for std::io::Error {
            #[inline]
            fn from(error: Error) -> Self {
                error.inner
            }
        }

        impl std::error::Error for Error {
            fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                Some(&self.inner)
            }
        }
    }
}
