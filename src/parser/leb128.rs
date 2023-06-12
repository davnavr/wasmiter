//! Functions for parsing integers in the
//! [*LEB128* format](https://webassembly.github.io/spec/core/binary/values.html#integers).

use crate::bytes::{self, Bytes};
use crate::parser::{Error, Result, ResultExt};

// TODO: Expose specific parsers for use in benchmarks (#[cfg(bench)])?
mod default;

const CONTINUATION: u8 = 0b1000_0000u8;
const SIGN: u8 = 0b0100_0000u8;

#[cold]
#[inline(never)]
fn too_large<T>(signed: bool) -> Error {
    let signedness = if signed { "signed" } else { "unsigned" };

    crate::parser_bad_format!(
        "decoded value cannot fit into a {}-bit {signedness} integer",
        core::mem::size_of::<T>() / 8
    )
}

trait IntegerEncoding:
    From<u8> + Default + core::ops::BitOrAssign + core::ops::Shl<u8, Output = Self>
{
    /// The maximum number of bytes that a value is allowed to be encoded in.
    ///
    /// According to the
    /// [WebAssembly specification](https://webassembly.github.io/spec/core/binary/values.html#integers),
    /// this should be equal to `ceil(BITS / 7)`.
    const MAX_LENGTH: u8;

    /// The bit-width of the integer.
    const BITS: u8;

    /// A buffer to contain the bytes encoding the integer.
    ///
    /// Should have a length equal to `MAX_LENGTH`.
    type Buffer: AsRef<[u8]> + AsMut<[u8]> + Default + IntoIterator<Item = u8>;

    #[cold]
    #[inline(never)]
    fn buffer_overflowed() -> Error {
        crate::parser_bad_format!("integer can only be encoded in {} bytes", Self::MAX_LENGTH)
    }
}

impl IntegerEncoding for u32 {
    const MAX_LENGTH: u8 = 5;
    const BITS: u8 = 32;
    type Buffer = [u8; Self::MAX_LENGTH as usize];
}

impl IntegerEncoding for u64 {
    const MAX_LENGTH: u8 = 10;
    const BITS: u8 = 64;
    type Buffer = [u8; Self::MAX_LENGTH as usize];
}

trait SignedInteger: IntegerEncoding + core::ops::ShrAssign<u8> {}

impl IntegerEncoding for i32 {
    const MAX_LENGTH: u8 = 5;
    const BITS: u8 = 32;
    type Buffer = [u8; Self::MAX_LENGTH as usize];
}

impl SignedInteger for i32 {}

impl IntegerEncoding for i64 {
    const MAX_LENGTH: u8 = 10;
    const BITS: u8 = 64;
    type Buffer = [u8; Self::MAX_LENGTH as usize];
}

impl SignedInteger for i64 {}

fn unsigned<N: IntegerEncoding, B: Bytes>(offset: &mut u64, bytes: B) -> Result<N> {
    let mut buffer = N::Buffer::default();
    let mut value = N::default();
    let input = bytes.read_at(*offset, buffer.as_mut())?;

    let mut more = true;
    let mut i: u8 = 0u8;
    for byte in input.iter().copied() {
        let bits = byte & 0x7F;
        more = byte & 0x80 == 0x80;

        let shift = 7u8 * i;

        // Check for overflowing bits in last byte
        if i == N::MAX_LENGTH - 1 {
            let leading_zeroes = bits.leading_zeros() as u8;
            if leading_zeroes < 8 - (N::BITS - shift) {
                // Overflow, the number of value bits will not fit in the destination
                return Err(crate::parser_bad_format!(
                    "encoded value requires {} bits, which cannot fit in the destination",
                    shift + (8 - leading_zeroes)
                ));
            }
        }

        debug_assert!(shift <= N::BITS);

        value |= N::from(bits) << shift;
        i += 1;

        if !more {
            break;
        }
    }

    if more {
        return Err(N::buffer_overflowed());
    }

    *offset = offset
        .checked_add(u64::from(i))
        .ok_or_else(bytes::offset_overflowed)?;

    Ok(value)
}

fn signed<N: SignedInteger, B: Bytes>(offset: &mut u64, bytes: B) -> Result<N> {
    let mut buffer = N::Buffer::default();
    let mut value = N::default();
    let input = bytes.read_at(*offset, buffer.as_mut())?;

    let mut more = true;
    let mut i = 0u8;
    let mut has_sign = false;
    // Duplicated code from leb128_unsigned
    for byte in input.iter().copied() {
        let bits = byte & 0x7F;
        more = byte & 0x80 == 0x80;
        has_sign = bits & 0x40 == 0x40;

        let shift = 7u8 * i;

        // Check for overflowing bits in last byte
        if i == N::MAX_LENGTH - 1 {
            if !has_sign {
                let leading_zeroes = (bits & 0x3F).leading_zeros() as u8;
                if leading_zeroes < 8 - (N::BITS - shift) {
                    return Err(crate::parser_bad_format!(
                        "encoded positive value requires {} bits, which cannot fit in the destination",
                        shift + (8 - leading_zeroes)
                    ));
                }
            } else {
                // TODO: Is the amount to check for correct here?
                let leading_ones = (bits | 0xC0).leading_ones() as u8;
                if leading_ones > (N::BITS - shift) + 1 {
                    return Err(crate::parser_bad_format!(
                        "negative value cannot be encoded in {} bits",
                        N::BITS
                    ));
                }
            }
        }

        debug_assert!(shift <= N::BITS);

        value |= N::from(bits & 0x7F) << shift;
        i += 1;

        if !more {
            break;
        }
    }

    if more {
        return Err(N::buffer_overflowed());
    }

    if has_sign {
        let mut sign_mask = N::from(1u8) << (N::BITS - 1);

        if i < N::MAX_LENGTH {
            // Right shift fills with sign
            sign_mask >>= N::BITS - (i * 7) - 1;
        }

        value |= sign_mask;
    }

    *offset = offset
        .checked_add(u64::from(i))
        .ok_or_else(bytes::offset_overflowed)?;

    Ok(value)
}

/// Attempts to a parse an unsigned 32-bit integer encoded in
/// [*LEB128* format](https://webassembly.github.io/spec/core/binary/values.html#integers).
pub fn u32<B: Bytes>(offset: &mut u64, bytes: B) -> Result<u32> {
    // TODO: less branches variant? have buffer = [0u8; 5] // 0 means end, even if peek does not fill buffer completely
    // copy-paste code that reads from buffer[0], buffer[1], etc. Only return at the very end?
    unsigned(offset, bytes).context("could not parse u32")
}

/// Attempts to parse a [`u32`](prim@u32) in *LEB128* format, interpreting the result as a
/// [`usize`](prim@usize).
///
/// This method is meant to parse
/// [vector lengths](https://webassembly.github.io/spec/core/binary/conventions.html#vectors),
/// which the specification currently limits to a 32-bit amount.
///
/// See [`leb128::u32`](self::u32) for more information.
pub fn usize<B: Bytes>(offset: &mut u64, bytes: B) -> Result<usize> {
    let length = unsigned::<u32, B>(offset, bytes).context("could not parse length")?;
    usize::try_from(length).map_err(|_| crate::parser_bad_format!("length ({length}) is too large"))
}

/// Attempts to a parse an unsigned 64-bit integer encoded in the
/// [*LEB128* format](https://webassembly.github.io/spec/core/binary/values.html#integers).
pub fn u64<B: Bytes>(offset: &mut u64, bytes: B) -> Result<u64> {
    unsigned(offset, bytes).context("could not parse u64")
}

/// Attempts to parse a signed 32-bit integer encoded in the
/// [*LEB128* format](https://webassembly.github.io/spec/core/binary/values.html#integers).
pub fn s32<B: Bytes>(offset: &mut u64, bytes: B) -> Result<i32> {
    default::s32_impl(offset, bytes).context("could not parse signed 32-bit integer")
}

/// Attempts to parse a signed 64-bit integer encoded in the
/// [*LEB128* format](https://webassembly.github.io/spec/core/binary/values.html#integers).
pub fn s64<B: Bytes>(offset: &mut u64, bytes: B) -> Result<i64> {
    signed(offset, bytes).context("could not parse s64")
}
