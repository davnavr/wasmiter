//! Low-level types and functions for parsing.

mod error;
mod result_ext;
mod simple_parse;
// mod vector;

pub mod leb128;

pub use error::{Context, Error, ErrorKind};
pub use result_ext::ResultExt;
pub use simple_parse::SimpleParse;
// pub use vector::{Sequence, Vector};

/// Result type used when parsing input.
pub type Result<T> = core::result::Result<T, Error>;

#[macro_export]
#[doc(hidden)]
macro_rules! parser_bad_format {
    ($($arg:tt)*) => {{
        let err: $crate::parser::Error;

        #[cfg(not(feature = "alloc"))]
        {
            // Disable warnings for unused variables
            let _ = |f: &mut core::fmt::Formatter<'_>| core::write!(f, $($arg)*);
            err = $crate::parser::Error::bad_format();
        }

        #[cfg(feature = "alloc")]
        {
            err = $crate::parser::Error::bad_format().with_context(alloc::format!($($arg)*));
        }

        err
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! parser_bad_input {
    ($error:expr, $($arg:tt)*) => {{
        #[cfg(not(feature = "alloc"))]
        let err;
        #[cfg(feature = "alloc")]
        let mut err;

        err = <Error as From<input::Error>>::from($error);

        #[cfg(feature = "alloc")]
        {
            err = err.with_context(alloc::format!($($arg)*));
        }

        #[cfg(not(feature = "alloc"))]
        {
            // Disable warning for unused expression $error
            let _ = $error;
            let _ = |f: &mut core::fmt::Formatter| core::write!(f, $($arg)*);
        }

        err
    }};
}

/// Trait for parsers.
pub trait Parse {
    /// The result of the parser.
    type Output;

    /// Parses the given input.
    fn parse<B: crate::bytes::Bytes>(&mut self, input: &mut Decoder<B>) -> Result<Self::Output>;
}
