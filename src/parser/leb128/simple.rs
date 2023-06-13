//! Simple *LEB128* parsers written in Rust, without making explicit use of any architecture
//! specific intrinsics (such as those in [`core::arch::*`](core::arch)).
//!
//! This API is **an implementation detail and is not stable**, it is only made public for use in
//! benchmarks.

use crate::{
    bytes::{offset_overflowed, Bytes},
    parser::Result,
};

macro_rules! unsigned {
    ($(
        $(#[$meta:meta])*
        $vis:vis fn $name:ident => $ty:ty;
    )*) => {$(
        $(#[$meta])*
        $vis fn $name<B: Bytes>(offset: &mut u64, bytes: B) -> Result<$ty> {
            const BITS: u8 = <$ty>::BITS as u8;
            const MAX_BYTE_WIDTH: u8 = (BITS / 7) + 1;

            let mut buffer = [0u8; MAX_BYTE_WIDTH as usize];
            let mut value: $ty = 0;
            let input: &[u8] = bytes.read_at(*offset, &mut buffer)?;

            let mut more = true;
            let mut i: u8 = 0u8;
            for byte in input.iter().copied() {
                let bits = byte & 0x7F;
                more = byte & 0x80 == 0x80;

                let shift = 7u8 * i;

                // Check for overflowing bits in last byte
                if i == MAX_BYTE_WIDTH - 1 {
                    let leading_zeroes = bits.leading_zeros() as u8;
                    if leading_zeroes < 8 - (BITS - shift) {
                        // Overflow, the number of value bits will not fit in the destination
                        return Err(super::too_large::<$ty>(false));
                    }
                }

                debug_assert!(shift <= BITS);

                value |= bits as $ty << shift;
                i += 1;

                if !more {
                    break;
                }
            }

            if more {
                return Err(super::bad_continuation(&input));
            }

            *offset = offset
                .checked_add(u64::from(i))
                .ok_or_else(offset_overflowed)?;

            Ok(value)
        }
    )*};
}

unsigned! {
    #[doc(hidden)]
    pub fn u32 => u32;
    #[doc(hidden)]
    pub fn u64 => u64;
}

#[doc(hidden)]
pub fn s32<B: Bytes>(offset: &mut u64, bytes: B) -> Result<i32> {
    let mut destination = 0u32;
    let mut buffer = [0u8; 5];
    let input: &[u8] = bytes.read_at(*offset, &mut buffer)?;
    let mut remaining = input.iter().copied();

    // Read the first 4 bytes
    for shift_amount in (0u8..4).map(|i| i * 7) {
        let b = remaining
            .next()
            .ok_or_else(|| super::bad_continuation(input))?;

        destination |= ((b & 0x7Fu8) as u32) << shift_amount;

        *offset = offset.checked_add(1).ok_or_else(offset_overflowed)?;

        if b & super::CONTINUATION == 0 {
            // Sign extend the value
            let sign = (((b & super::SIGN) as u32).rotate_right(7) as i32) >> (24u8 - shift_amount);

            return Ok(destination as i32 | sign);
        }
    }

    // Read the last byte
    let last = remaining
        .next()
        .ok_or_else(|| super::bad_continuation(input))?;

    destination |= ((last & 0b1111) as u32) << 28;

    *offset = offset.checked_add(1).ok_or_else(offset_overflowed)?;

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
        let b = remaining
            .next()
            .ok_or_else(|| super::bad_continuation(input))?;

        destination |= ((b & 0x7Fu8) as u64) << shift_amount;

        *offset = offset.checked_add(1).ok_or_else(offset_overflowed)?;

        if b & super::CONTINUATION == 0 {
            // Sign extend the value
            let sign = (((b & super::SIGN) as u64).rotate_right(7) as i64) >> (56u8 - shift_amount);

            return Ok(destination as i64 | sign);
        }
    }

    // Read the last byte
    let last = remaining
        .next()
        .ok_or_else(|| super::bad_continuation(input))?;

    destination |= ((last & 1) as u64) << 63;

    *offset = offset.checked_add(1).ok_or_else(offset_overflowed)?;

    if matches!(last & 0b1111_1110, 0 | 0b0111_1110) {
        Ok(destination as i64)
    } else {
        Err(super::too_large::<i64>(true))
    }
}
