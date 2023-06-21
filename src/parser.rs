//! Low-level types and functions for parsing.

use crate::input::Input;

mod ascending_order;
mod error;
mod mixed_error;
mod offset;
mod result_ext;
mod vector;

pub mod leb128;
pub mod name;

pub(crate) use ascending_order::AscendingOrder;
pub(crate) use error::{Context, ErrorKind};
pub(crate) use result_ext::ResultExt;

pub use error::Error;
pub use mixed_error::{MixedError, MixedResult};
pub use offset::Offset;
pub use vector::Vector;

/// Result type used when parsing bytes from an [`Input`].
pub type Parsed<T> = Result<T, Error>;

#[inline]
pub(crate) fn bytes_exact<I: Input>(offset: &mut u64, input: I, buffer: &mut [u8]) -> Parsed<()> {
    let length = buffer.len();
    input.read_exact(offset, buffer).with_context(|| {
        move |f| {
            write!(f, "expected {length} byte")?;
            if length != 1 {
                core::fmt::Write::write_char(f, 's')?;
            }
            Ok(())
        }
    })
}

#[inline]
pub(crate) fn byte_array<I: Input, const N: usize>(offset: &mut u64, input: I) -> Parsed<[u8; N]> {
    let mut array = [0u8; N];
    bytes_exact(offset, input, array.as_mut_slice())?;
    Ok(array)
}

#[inline]
pub(crate) fn one_byte<I: Input>(offset: &mut u64, input: I) -> Parsed<Option<u8>> {
    let mut byte = 0u8;
    let width = input
        .read(offset, core::slice::from_mut(&mut byte))
        .context("could not read a byte")?;

    Ok(if width == 1 { Some(byte) } else { None })
}

#[inline]
pub(crate) fn one_byte_exact<I: Input>(offset: &mut u64, input: I) -> Parsed<u8> {
    let mut value = 0u8;
    bytes_exact(offset, input, core::slice::from_mut(&mut value))?;
    Ok(value)
}
