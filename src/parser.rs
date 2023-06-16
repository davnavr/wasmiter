//! Low-level types and functions for parsing.

use crate::input::Input;

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

        err = <$crate::parser::Error as From<$crate::input::Error>>::from($error);

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

#[inline]
pub(crate) fn bytes<'b, I: Input>(
    offset: &mut u64,
    input: I,
    buffer: &'b mut [u8],
) -> Result<&'b mut [u8]> {
    let length = buffer.len();
    input
        .read(offset, buffer)
        .map_err(|e| parser_bad_input!(e, "could not read {} bytes", length))
}

#[inline]
pub(crate) fn bytes_exact<I: Input>(offset: &mut u64, input: I, buffer: &mut [u8]) -> Result<()> {
    input
        .read_exact(offset, buffer)
        .map_err(|e| parser_bad_input!(e, "expected {} bytes", buffer.len()))
}

#[inline]
pub(crate) fn byte_array<I: Input, const N: usize>(offset: &mut u64, input: I) -> Result<[u8; N]> {
    let mut array = [0u8; N];
    bytes_exact(offset, input, array.as_mut_slice())?;
    Ok(array)
}

#[inline]
pub(crate) fn one_byte<I: Input>(offset: &mut u64, input: I) -> Result<Option<u8>> {
    Ok(if let [value] = self::bytes(offset, input, &mut [0u8])? {
        Some(*value)
    } else {
        None
    })
}

#[inline]
pub(crate) fn one_byte_exact<I: Input>(offset: &mut u64, input: I) -> Result<u8> {
    let mut value = 0u8;
    bytes_exact(offset, input, core::slice::from_mut(&mut value))?;
    Ok(value)
}
