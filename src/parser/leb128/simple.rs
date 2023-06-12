//! Simple *LEB128* parsers written in Rust, without making explicit use of any architecture
//! specific intrinsics (such as those in [`core::arch::*`](core::arch))

use crate::{bytes::Bytes, parser::Result};

pub(super) fn s32<B: Bytes>(offset: &mut u64, bytes: B) -> Result<i32> {
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
        *offset += 1;

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
    *offset += 1;

    if matches!(last & 0b1111_0000, 0 | 0b0111_0000) {
        Ok(destination as i32)
    } else {
        Err(super::too_large::<i32>(true))
    }
}

pub(super) fn s64<B: Bytes>(offset: &mut u64, bytes: B) -> Result<i64> {
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
        *offset += 1;

        if b & super::CONTINUATION == 0 {
            // Sign extend the value
            let sign = (((b & super::SIGN) as u64).rotate_right(7) as i64) >> (63u8 - shift_amount);
            return Ok(destination as i64 | sign);
        }
    }

    // Read the last byte
    let last = remaining
        .next()
        .ok_or_else(|| super::bad_continuation(input))?;

    destination |= ((last & 1) as u64) << 63;
    *offset += 1;

    if matches!(last & 0b1111_1110, 0 | 0b0111_1110) {
        Ok(destination as i64)
    } else {
        Err(super::too_large::<i64>(true))
    }
}
