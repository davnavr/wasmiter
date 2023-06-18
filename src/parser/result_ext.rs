use crate::parser::{Context, Error, Result};
use core::fmt::Formatter;

/// Provides helper methods to add additional context to [`Error`]s.
///
/// If the `std` and `alloc` features are not specified, then these methods do nothing.
pub(crate) trait ResultExt<T> {
    /// Attaches the given [`Context`] to the [`Result<T>`], if it is an error.
    fn context<C: Into<Context>>(self, context: C) -> Result<T>
    where
        Self: Sized;

    fn with_context<F, C>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> C,
        C: Fn(&mut Formatter) -> core::fmt::Result + Send + Sync + 'static;
}

impl<T, E: Into<Error>> ResultExt<T> for core::result::Result<T, E> {
    #[inline]
    fn context<C: Into<Context>>(self, context: C) -> Result<T>
    where
        Self: Sized,
    {
        match self {
            Ok(ok) => Ok(ok),
            Err(err) => Err(err.into().with_context(context.into())),
        }
    }

    #[inline]
    fn with_context<F, C>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> C,
        C: Fn(&mut Formatter) -> core::fmt::Result + Send + Sync + 'static,
    {
        match self {
            Ok(ok) => Ok(ok),
            Err(e) => {
                cfg_if::cfg_if! {
                    if #[cfg(feature = "alloc")] {
                        #[cold]
                        #[inline(never)]
                        fn make_context_closure<C>(c: C) -> Context
                        where
                            C: Fn(&mut Formatter) -> core::fmt::Result + Send + Sync + 'static,
                        {
                            Context::from_closure(c)
                        }

                        Err(e.into().with_context(make_context_closure(f())))
                    } else {
                        Err(e.into())
                    }
                }
            }
        }
    }
}
