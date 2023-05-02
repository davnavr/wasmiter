use crate::parser::input;
use core::fmt::{Debug, Display, Formatter};

#[cfg(feature = "backtrace")]
use std::backtrace::Backtrace;

#[cfg(feature = "alloc")]
use alloc::{borrow::Cow, string::String};

/// Specifies what kind of error occured during parsing.
#[derive(Debug)]
#[non_exhaustive]
pub enum ErrorKind {
    /// An I/O error occured or an attempt to retrieve bytes from the [`Input`](input::Input)
    /// failed.
    BadInput(input::Error),
    /// The input is malformed.
    InvalidFormat,
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::BadInput(e) => Display::fmt(e, f),
            Self::InvalidFormat => Display::fmt("input was malformed", f),
        }
    }
}

impl From<input::Error> for ErrorKind {
    fn from(error: input::Error) -> Self {
        Self::BadInput(error)
    }
}

/// Adds additional information to an [`Error`].
pub struct Context {
    // TODO: Make an enum ContextKind?
    #[cfg(feature = "alloc")]
    message: Cow<'static, str>,
    #[cfg(not(feature = "alloc"))]
    message: &'static str,
}

#[cfg(feature = "alloc")]
impl From<Cow<'static, str>> for Context {
    #[inline]
    fn from(message: Cow<'static, str>) -> Self {
        Self { message }
    }
}

impl From<&'static str> for Context {
    #[inline]
    fn from(message: &'static str) -> Self {
        #[cfg(feature = "alloc")]
        return Self::from(Cow::Borrowed(message));

        #[cfg(not(feature = "alloc"))]
        return Self { message };
    }
}

#[cfg(feature = "alloc")]
impl From<String> for Context {
    #[inline]
    fn from(message: String) -> Self {
        Self::from(Cow::Owned(message))
    }
}

impl Display for Context {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&self.message, f)
    }
}

impl Debug for Context {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self.message, f)
    }
}

#[cfg(feature = "alloc")]
struct ErrorInner {
    kind: ErrorKind,
    context: alloc::vec::Vec<Context>,
    #[cfg(feature = "backtrace")]
    backtrace: Backtrace,
}

/// Describes an error that occured during parsing.
#[repr(transparent)]
pub struct Error {
    #[cfg(feature = "alloc")]
    inner: alloc::boxed::Box<ErrorInner>,
    #[cfg(not(feature = "alloc"))]
    kind: ErrorKind,
}

impl Error {
    fn new(kind: ErrorKind) -> Self {
        #[cfg(feature = "alloc")]
        return Self {
            inner: alloc::boxed::Box::new(ErrorInner {
                kind,
                context: Default::default(),
                #[cfg(feature = "backtrace")]
                backtrace: Backtrace::capture(),
            }),
        };

        #[cfg(not(feature = "alloc"))]
        return Self { kind };
    }

    /// Gets a [`Backtrace`] describing where in the code the error occured.
    #[cfg(feature = "backtrace")]
    #[inline]
    pub fn backtrace(&self) -> &Backtrace {
        &self.inner.backtrace
    }

    /// Gets the kind of error that occured.
    #[inline]
    pub fn kind(&self) -> &ErrorKind {
        #[cfg(feature = "alloc")]
        return &self.inner.kind;

        #[cfg(not(feature = "alloc"))]
        return &self.kind;
    }

    #[inline]
    fn context_list(&self) -> &[Context] {
        #[cfg(feature = "alloc")]
        return &self.inner.context;

        #[cfg(not(feature = "alloc"))]
        return &[];
    }
}

impl From<input::Error> for Error {
    #[inline]
    fn from(error: input::Error) -> Self {
        Self::new(ErrorKind::from(error))
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut s = f.debug_struct("Error");

        s.field("kind", self.kind());
        s.field("context", &self.context_list());

        #[cfg(feature = "backtrace")]
        {
            s.field("backtrace", self.backtrace());
        }

        s.finish()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "{}", self.kind())?;
        for context in self.context_list().iter() {
            writeln!(f, "- {context}")?;
        }

        #[cfg(feature = "backtrace")]
        if !f.alternate() && self.backtrace().status() == std::backtrace::BacktraceStatus::Captured
        {
            writeln!(f, "with backtrace:")?;
            writeln!(f, "{}", self.backtrace())?;
        }

        Ok(())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        match self.kind() {
            ErrorKind::BadInput(error) => Some(error),
            _ => None,
        }
    }
}
