use crate::parser::{Context, Error, Result};

mod sealed {
    pub trait Sealed {}

    impl<T, E: Into<crate::parser::Error>> Sealed for core::result::Result<T, E> {}
}

/// Provides helper methods to add additional context to [`Error`]s.
///
/// If the `std` and `alloc` features are not specified, then these methods do nothing.
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

impl<T, E: Into<Error>> ResultExt<T> for core::result::Result<T, E> {
    fn with_context<C: Into<Context>, F: FnOnce() -> C>(self, f: F) -> Result<T> {
        #[cold]
        #[inline(never)]
        fn create_error<C: Into<Context>>(e: impl Into<Error>, f: impl FnOnce() -> C) -> Error {
            let mut err = e.into();
            err.context(f());
            err
        }

        match self {
            Ok(value) => Ok(value),
            Err(e) => Err(create_error(e, f)),
        }
    }
}
