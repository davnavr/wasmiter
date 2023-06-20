//! Safe integer conversion functions.

#![allow(clippy::cast_possible_truncation)]

#[inline(always)]
pub(crate) const fn u32_to_usize(i: u32) -> usize {
    // usize::BITS >= 32
    i as usize
}
