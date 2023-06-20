use crate::parser::{self, Error};
use core::fmt::Display;

/// Result type used when parsing either fails or some other error occurs.
pub type MixedResult<T, E> = Result<T, MixedError<E>>;

/// Error type used in some parser functions that accept closures, allowing both a [`parser::Error`] and a user-defined error to be returned.
#[derive(Debug)]
pub enum MixedError<E = core::convert::Infallible> {
    /// A parser error occured.
    Parser(Error),
    /// Some other error occured.
    User(E),
}

const _SIZE_CHECK: [(); core::mem::size_of::<Error>()] =
    [(); core::mem::size_of::<Option<MixedError>>()];

impl<E> MixedError<E> {
    #[inline]
    pub(crate) fn map_parser_err(self, f: impl FnOnce(Error) -> Error) -> Self {
        match self {
            Self::Parser(e) => Self::Parser(f(e)),
            Self::User(e) => Self::User(e),
        }
    }
}

impl<T, E> parser::ResultExt<Result<T, MixedError<E>>> for Result<T, MixedError<E>> {
    #[inline]
    fn context<C: Into<parser::Context>>(self, context: C) -> Self
    where
        Self: Sized,
    {
        match self {
            Ok(ok) => Ok(ok),
            Err(err) => Err(err.map_parser_err(move |e| e.with_context(context.into()))),
        }
    }

    #[inline]
    fn with_context<F, C>(self, f: F) -> Self
    where
        F: FnOnce() -> C,
        C: Fn(&mut core::fmt::Formatter) -> core::fmt::Result + Send + Sync + 'static,
    {
        match self {
            Ok(ok) => Ok(ok),
            Err(err) => Err(err
                .map_parser_err(move |e| e.with_context(parser::Context::from_closure_cold(f())))),
        }
    }

    #[inline]
    fn with_location_context(self, description: &'static str, offset: u64) -> Self {
        match self {
            Ok(ok) => Ok(ok),
            Err(err) => {
                Err(err.map_parser_err(|e| e.with_location_context_cold(description, offset)))
            }
        }
    }
}

impl<E> From<Error> for MixedError<E> {
    #[inline]
    fn from(error: Error) -> Self {
        Self::Parser(error)
    }
}

impl<E> From<crate::input::Error> for MixedError<E> {
    #[inline]
    fn from(error: crate::input::Error) -> Self {
        Self::Parser(error.into())
    }
}

impl From<MixedError> for Error {
    fn from(error: MixedError) -> Self {
        match error {
            MixedError::Parser(inner) => inner,
            MixedError::User(bad) => match bad {},
        }
    }
}

impl<E: Display> Display for MixedError<E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            MixedError::Parser(e) => Display::fmt(e, f),
            MixedError::User(e) => Display::fmt(e, f),
        }
    }
}

#[cfg(feature = "std")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "std")))]
impl<E: core::fmt::Debug + Display> std::error::Error for MixedError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Parser(e) => Some(e),
            Self::User(_) => None,
        }
    }
}
