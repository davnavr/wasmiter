//! Simple *LEB128* parsers written in Rust, without making explicit use of any architecture
//! specific intrinsics (such as those in [`core::arch::*`](core::arch)).
//!
//! This API is **an implementation detail and is not stable**, it is only made public for use in
//! benchmarks.

use crate::{bytes::Bytes, parser::Result};

#[inline]
fn increment_offset(offset: &mut u64) -> Result<()> {
    if let Some(incremented) = offset.checked_add(1) {
        *offset = incremented;
        Ok(())
    } else {
        Err(crate::bytes::offset_overflowed().into())
    }
}

#[inline]
fn next_byte<'a>(
    input: &'a [u8],
    remaining: &mut core::iter::Copied<core::slice::Iter<'a, u8>>,
) -> Result<u8> {
    if let Some(byte) = remaining.next() {
        Ok(byte)
    } else {
        Err(super::bad_continuation(input))
    }
}

macro_rules! unsigned {
    ($(fn $name:ident => $ty:ty;)*) => {$(
        pub fn $name<B: Bytes>(offset: &mut u64, bytes: B) -> Result<$ty> {
            const BITS: u8 = <$ty>::BITS as u8;
            const MAX_BYTE_WIDTH: u8 = (BITS / 7) + 1;

            let mut buffer = [0u8; MAX_BYTE_WIDTH as usize];
            let input = bytes.read_at(*offset, &mut buffer)?;

            let mut remaining = input.iter().copied();
            let mut value: $ty = 0;

            for shift in (0u8..MAX_BYTE_WIDTH).map(|i| i * 7) {
                let byte = next_byte(&input, &mut remaining)?;

                // Check for overflow
                if shift == (MAX_BYTE_WIDTH - 1) * 7 && (byte & (0xFFu8 << (BITS - ((BITS / 7) * 7))) != 0) {
                    return Err(super::too_large::<$ty>(false));
                }

                value |= ((byte & super::VALUE_MASK) as $ty) << shift;

                increment_offset(offset)?;

                if byte & super::CONTINUATION == 0 {
                    break;
                }
            }

            Ok(value)
        }
    )*};
}

unsigned! {
    fn u32 => u32;
    fn u64 => u64;
}

#[doc(hidden)]
pub fn s32<B: Bytes>(offset: &mut u64, bytes: B) -> Result<i32> {
    let mut destination = 0u32;
    let mut buffer = [0u8; 5];
    let input: &[u8] = bytes.read_at(*offset, &mut buffer)?;
    let mut remaining = input.iter().copied();

    // Read the first 4 bytes
    for shift_amount in (0u8..4).map(|i| i * 7) {
        let byte = next_byte(input, &mut remaining)?;

        destination |= ((byte & 0x7Fu8) as u32) << shift_amount;

        increment_offset(offset)?;

        if byte & super::CONTINUATION == 0 {
            // Sign extend the value
            let sign =
                (((byte & super::SIGN) as u32).rotate_right(7) as i32) >> (24u8 - shift_amount);

            return Ok(destination as i32 | sign);
        }
    }

    // Read the last byte
    let last = remaining
        .next()
        .ok_or_else(|| super::bad_continuation(input))?;

    destination |= ((last & 0b1111) as u32) << 28;

    increment_offset(offset)?;

    if matches!(last & 0b1111_0000, 0 | 0b0111_0000) {
        Ok(destination as i32)
    } else {
        Err(super::too_large::<i32>(true))
    }
}

#[doc(hidden)]
pub fn s64<B: Bytes>(offset: &mut u64, bytes: B) -> Result<i64> {
    let mut destination = 0u64;
    let mut buffer = [0u8; 10];
    let input: &[u8] = bytes.read_at(*offset, &mut buffer)?;
    let mut remaining = input.iter().copied();

    // Read the first 9 bytes
    for shift_amount in (0u8..9).map(|i| i * 7) {
        let byte = next_byte(input, &mut remaining)?;

        destination |= ((byte & 0x7Fu8) as u64) << shift_amount;

        increment_offset(offset)?;

        if byte & super::CONTINUATION == 0 {
            // Sign extend the value
            let sign =
                (((byte & super::SIGN) as u64).rotate_right(7) as i64) >> (56u8 - shift_amount);

            return Ok(destination as i64 | sign);
        }
    }

    // Read the last byte
    let last = remaining
        .next()
        .ok_or_else(|| super::bad_continuation(input))?;

    destination |= ((last & 1) as u64) << 63;

    increment_offset(offset)?;

    if matches!(last & 0b1111_1110, 0 | 0b0111_1110) {
        Ok(destination as i64)
    } else {
        Err(super::too_large::<i64>(true))
    }
}
