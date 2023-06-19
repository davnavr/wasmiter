//! Functions for parsing integers in the
//! [*LEB128* format](https://webassembly.github.io/spec/core/binary/values.html#integers).

use crate::input::Input;
use crate::parser::{Context, Error, ErrorKind, Parsed, ResultExt as _};

// Implementation modules, used in benchmarks

#[doc(hidden)]
pub mod simple;

// TODO: Add modules for platform-specific SIMD acceleration

use simple as implementation;

const CONTINUATION: u8 = 0b1000_0000;
const VALUE_MASK: u8 = 0b0111_1111;
const SIGN: u8 = 0b0100_0000;

#[cold]
#[inline(never)]
fn too_large<T>(signed: bool) -> Error {
    Error::new(ErrorKind::VarLenIntTooLarge {
        bits: (core::mem::size_of::<T>() * 8) as u8,
        signed,
    })
}

#[cold]
#[inline(never)]
fn bad_continuation(bytes: &[u8]) -> Error {
    let length = bytes.len();
    let mut buffer = [0u8; 16];
    buffer[..length].copy_from_slice(bytes);

    Error::new(ErrorKind::InvalidFormat).with_context(Context::from_closure(move |f| {
        write!(
            f,
            "continuation flag was set in integer {:#?}, but no more bytes remain in the input",
            crate::input::HexDump::from(&buffer[..length])
        )
    }))
}

/// Attempts to a parse an unsigned 32-bit integer encoded in
/// [*LEB128* format](https://webassembly.github.io/spec/core/binary/values.html#integers).
pub fn u32<I: Input>(offset: &mut u64, input: I) -> Parsed<u32> {
    implementation::u32(offset, input).context("could not parse unsigned 32-bit integer")
}

/// Attempts to parse a [`u32`](prim@u32) in *LEB128* format, interpreting the result as a
/// [`usize`](prim@usize).
///
/// This method is meant to parse
/// [vector lengths](https://webassembly.github.io/spec/core/binary/conventions.html#vectors),
/// which the specification currently limits to a 32-bit amount.
///
/// See [`leb128::u32`](self::u32) for more information.
pub fn usize<I: Input>(offset: &mut u64, input: I) -> Parsed<usize> {
    #[inline(never)]
    #[cold]
    fn length_too_large(length: u32) -> Error {
        Error::new(ErrorKind::InvalidFormat).with_context(Context::from_closure(move |f| write!(f, "parsed length ({length}) is too large, parsing WebAssembly in a 16-bit environment is not recommended")))
    }

    let length = self::u32(offset, input).context("could not parse length")?;
    if let Ok(parsed) = usize::try_from(length) {
        Ok(parsed)
    } else {
        Err(length_too_large(length))
    }
}

/// Attempts to a parse an unsigned 64-bit integer encoded in the
/// [*LEB128* format](https://webassembly.github.io/spec/core/binary/values.html#integers).
pub fn u64<I: Input>(offset: &mut u64, input: I) -> Parsed<u64> {
    implementation::u64(offset, input).context("could not parse unsigned 64-bit integer")
}

/// Attempts to parse a signed 32-bit integer encoded in the
/// [*LEB128* format](https://webassembly.github.io/spec/core/binary/values.html#integers).
pub fn s32<I: Input>(offset: &mut u64, input: I) -> Parsed<i32> {
    implementation::s32(offset, input).context("could not parse signed 32-bit integer")
}

/// Attempts to parse a signed 64-bit integer encoded in the
/// [*LEB128* format](https://webassembly.github.io/spec/core/binary/values.html#integers).
pub fn s64<I: Input>(offset: &mut u64, input: I) -> Parsed<i64> {
    implementation::s64(offset, input).context("could not parse signed 64-bit integer")
}
