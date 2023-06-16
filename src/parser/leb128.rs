//! Functions for parsing integers in the
//! [*LEB128* format](https://webassembly.github.io/spec/core/binary/values.html#integers).

use crate::input::Input;
use crate::parser::{Error, Result, ResultExt as _};

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
    let signedness = if signed { "signed" } else { "unsigned" };

    crate::parser_bad_format!(
        "decoded value cannot fit into a {}-bit {signedness} integer",
        core::mem::size_of::<T>() / 8
    )
}

#[cold]
#[inline(never)]
fn bad_continuation(bytes: &[u8]) -> Error {
    crate::parser_bad_format!(
        "continuation flag was set in integer {:#?}, but no more bytes remain in the input",
        crate::input::HexDump::from(bytes)
    )
}

/// Attempts to a parse an unsigned 32-bit integer encoded in
/// [*LEB128* format](https://webassembly.github.io/spec/core/binary/values.html#integers).
pub fn u32<I: Input>(offset: &mut u64, input: I) -> Result<u32> {
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
pub fn usize<I: Input>(offset: &mut u64, input: I) -> Result<usize> {
    let length = self::u32(offset, input).context("could not parse length")?;
    usize::try_from(length).map_err(|_| crate::parser_bad_format!("length ({length}) is too large"))
}

/// Attempts to a parse an unsigned 64-bit integer encoded in the
/// [*LEB128* format](https://webassembly.github.io/spec/core/binary/values.html#integers).
pub fn u64<I: Input>(offset: &mut u64, input: I) -> Result<u64> {
    implementation::u64(offset, input).context("could not parse unsigned 64-bit integer")
}

/// Attempts to parse a signed 32-bit integer encoded in the
/// [*LEB128* format](https://webassembly.github.io/spec/core/binary/values.html#integers).
pub fn s32<I: Input>(offset: &mut u64, input: I) -> Result<i32> {
    implementation::s32(offset, input).context("could not parse signed 32-bit integer")
}

/// Attempts to parse a signed 64-bit integer encoded in the
/// [*LEB128* format](https://webassembly.github.io/spec/core/binary/values.html#integers).
pub fn s64<I: Input>(offset: &mut u64, input: I) -> Result<i64> {
    implementation::s64(offset, input).context("could not parse signed 64-bit integer")
}
