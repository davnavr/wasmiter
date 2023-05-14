//! Low-level types and functions for parsing.

mod error;
mod offset;
mod result_ext;
mod simple_parse;
mod vector;

pub mod leb128;

pub use error::{Context, Error, ErrorKind};
pub use offset::Offset;
pub use result_ext::ResultExt;
pub use simple_parse::SimpleParse;
pub use vector::{vector, Sequence, Vector};

use crate::bytes::Bytes;

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

        err = <$crate::parser::Error as From<$crate::bytes::Error>>::from($error);

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
    fn parse<B: crate::bytes::Bytes>(&mut self, offset: &mut u64, bytes: B)
        -> Result<Self::Output>;
}

#[inline]
pub(crate) fn bytes<'b, B: Bytes>(
    offset: &mut u64,
    bytes: B,
    buffer: &'b mut [u8],
) -> Result<&'b mut [u8]> {
    let length = buffer.len();
    bytes
        .read(offset, buffer)
        .map_err(|e| parser_bad_input!(e, "could not read {} bytes", length))
}

#[inline]
pub(crate) fn bytes_exact<B: Bytes>(offset: &mut u64, bytes: B, buffer: &mut [u8]) -> Result<()> {
    bytes
        .read_exact(offset, buffer)
        .map_err(|e| parser_bad_input!(e, "expected {} bytes", buffer.len()))
}

#[inline]
pub(crate) fn one_byte<B: Bytes>(offset: &mut u64, bytes: B) -> Result<Option<u8>> {
    Ok(if let [value] = self::bytes(offset, bytes, &mut [0u8])? {
        Some(*value)
    } else {
        None
    })
}

#[inline]
pub(crate) fn one_byte_exact<B: Bytes>(offset: &mut u64, bytes: B) -> Result<u8> {
    let mut value = 0u8;
    bytes_exact(offset, bytes, core::slice::from_mut(&mut value))?;
    Ok(value)
}

// pub(crate) fn skip_exact(&mut self, amount: u64) -> Result<()> {
//     let actual = self
//         .input
//         .read(amount)
//         .map_err(|e| parser_bad_input!(e, "could not read {amount} bytes"))?;
//
//     if amount != actual {
//         return Err(parser_bad_format!(
//             "attempt to read {amount} bytes, but read {actual} before reaching end of input"
//         ));
//     }
//
//     Ok(())
// }

/// Parses an UTF-8 string
/// [name](https://webassembly.github.io/spec/core/binary/values.html#names). Allocates extra space
/// into the given `buffer`, and appends to contents of the string to it.
pub fn name<'b, B: Bytes, U: crate::allocator::Buffer>(
    offset: &mut u64,
    bytes: &B,
    buffer: &'b mut U,
) -> Result<&'b mut str> {
    let length = leb128::usize(offset, bytes).context("string length")?;
    let destination = buffer.as_ref().len()..;
    buffer.grow(length);
    bytes_exact(offset, bytes, &mut buffer.as_mut()[destination]).context("string contents")?;
    core::str::from_utf8_mut(&mut buffer.as_mut()[destination])
        .map_err(|e| crate::parser_bad_format!("{e}"))
}
