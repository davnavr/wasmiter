//! Low-level types and functions for parsing.

use crate::bytes::Bytes;

mod ascending_order;
mod error;
mod offset;
mod result_ext;
mod vector;

pub mod leb128;
pub mod name;

pub(crate) use ascending_order::AscendingOrder;

pub use error::{Context, Error, ErrorKind};
pub use offset::Offset;
pub use result_ext::ResultExt;
pub use vector::Vector;

/// Result type used when parsing input.
pub type Result<T> = core::result::Result<T, Error>;

#[macro_export]
#[doc(hidden)]
macro_rules! parser_bad_format {
    ($($arg:tt)*) => {{
        cfg_if::cfg_if! {
            if #[cfg(feature = "alloc")] {
                let err = $crate::parser::Error::bad_format().with_context(alloc::format!($($arg)*));
            } else {
                // Disable warnings for unused variables
                let _ = |f: &mut core::fmt::Formatter<'_>| core::write!(f, $($arg)*);
                let err = $crate::parser::Error::bad_format();
            }
        }

        err
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! parser_bad_format_at_offset {
    ($location:literal @ $offset:expr $(, $($arg:tt)*)?) => {{
        let err = $crate::parser_bad_format!(concat!("at offset {:#X} in ", $location), $offset);

        $(
            cfg_if::cfg_if! {
                if #[cfg(feature = "alloc")] {
                    let err = err.with_context(alloc::format!($($arg)*));
                } else {
                    // Disable warnings for unused variables
                    let _ = |f: &mut core::fmt::Formatter<'_>| core::write!(f, $($arg)*);
                }
            }
        )?

        err
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! parser_bad_input {
    ($error:expr, $($arg:tt)*) => {{
        cfg_if::cfg_if! {
            if #[cfg(feature = "alloc")] {
                let mut err;
            } else {
                let err;
            }
        }

        err = <$crate::parser::Error as From<$crate::bytes::Error>>::from($error);

        cfg_if::cfg_if! {
            if #[cfg(feature = "alloc")] {
                err = err.with_context(alloc::format!($($arg)*));
            } else {
                // Disable warning for unused expression $error
                let _ = $error;
                let _ = |f: &mut core::fmt::Formatter| core::write!(f, $($arg)*);
            }
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
pub(crate) fn byte_array<B: Bytes, const N: usize>(offset: &mut u64, bytes: B) -> Result<[u8; N]> {
    let mut array = [0u8; N];
    bytes_exact(offset, bytes, array.as_mut_slice())?;
    Ok(array)
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
