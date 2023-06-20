use crate::parser::{self, Context, Parsed};
use core::fmt::Formatter;

/// Provides helper methods to add additional context to [`parser::Error`]s.
///
/// If the `std` and `alloc` features are not specified, then these methods do nothing.
pub(crate) trait ResultExt<R> {
    /// Attaches the given [`Context`] to the [`Result<T>`], if it is an error.
    fn context<C: Into<Context>>(self, context: C) -> R
    where
        Self: Sized;

    fn with_context<F, C>(self, f: F) -> R
    where
        F: FnOnce() -> C,
        C: Fn(&mut Formatter) -> core::fmt::Result + Send + Sync + 'static;

    fn with_location_context(self, description: &'static str, offset: u64) -> R;
}

impl<T, E: Into<parser::Error>> ResultExt<Parsed<T>> for Result<T, E> {
    #[inline]
    fn context<C: Into<Context>>(self, context: C) -> Parsed<T>
    where
        Self: Sized,
    {
        match self {
            Ok(ok) => Ok(ok),
            Err(e) => Err(e.into().with_context(context.into())),
        }
    }

    #[inline]
    fn with_context<F, C>(self, f: F) -> Parsed<T>
    where
        F: FnOnce() -> C,
        C: Fn(&mut Formatter) -> core::fmt::Result + Send + Sync + 'static,
    {
        match self {
            Ok(ok) => Ok(ok),
            Err(e) => Err(e.into().with_context(Context::from_closure_cold(f()))),
        }
    }

    #[inline]
    fn with_location_context(self, description: &'static str, offset: u64) -> Parsed<T> {
        match self {
            Ok(ok) => Ok(ok),
            Err(e) => Err(e.into().with_location_context_cold(description, offset)),
        }
    }
}
