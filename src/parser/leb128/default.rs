//! Simple *LEB128* parsers written in Rust, without making explicit use of any architecture
//! specific intrinsics (such as those in [`core::arch::*`](core::arch))

use crate::bytes::Bytes;

// Compiler should be able to make this branchless (except for the error check at the end)
pub(super) fn s32_impl<B: Bytes>(offset: &mut u64, bytes: B) -> crate::parser::Result<i32> {
    let mut buffer = 0u64.to_le_bytes();
    bytes.read_at(*offset, &mut buffer[0..5])?;

    let mut remaining = u64::from_le_bytes(buffer);
    let mut destination;

    #[inline]
    fn next_input(remaining: &mut u64, offset: &mut u64) {
        // First, check if the current byte has continuation and sign bits
        let masks = (*remaining as u8) & (super::CONTINUATION | super::SIGN);
        *remaining >>= 8;

        // Following code should do the following:
        if masks & super::CONTINUATION == 0 {
            // Only lower 8 bits matter, but it doesn't matter here if entire value is assigned
            // since future calls to next_input will also "overwrite" remaining garbage values here
            if masks & super::SIGN == 0 {
                *remaining = 0;
            } else {
                // Note that continuation bit is not set
                *remaining = 0b0111_1111;
            }
        } else {
            *offset += 1;
        }
    }

    // Read first byte
    destination = (remaining & 0x7F) as u32;
    *offset += 1; // TODO: Maybe for purpose of calculating offset update, could use a counter: u8 w/ trailing zeroes
    next_input(&mut remaining, offset);

    // Read next 3 bytes, should probably be unrolled
    for i in 1u32..4 {
        destination |= ((remaining & 0x7F) as u32) << (i * 7);
        next_input(&mut remaining, offset);
    }

    // Read final byte
    let last = remaining as u8;
    destination |= ((last & 0b1111u8) as u32) << 28;

    if matches!(last & 0b1111_0000, 0 | 0b0111_0000) {
        Ok(destination as i32)
    } else {
        Err(super::too_large::<i32>(true))
    }
}
