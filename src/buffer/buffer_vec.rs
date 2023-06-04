use crate::buffer::Buffer;
use alloc::vec::Vec;
use smallvec::SmallVec;

#[inline]
fn calculate_new_length_for_grow(current: usize, additional: usize) -> usize {
    current.checked_add(additional).unwrap_or(usize::MAX)
}

impl Buffer for Vec<u8> {
    #[inline]
    fn clear(&mut self) {
        <Vec<u8>>::clear(self)
    }

    #[inline]
    fn capacity(&self) -> Option<usize> {
        Some(<Vec<u8>>::capacity(self))
    }

    #[inline]
    fn grow(&mut self, length: usize) {
        <Vec<u8>>::resize(self, calculate_new_length_for_grow(self.len(), length), 0u8)
    }
}

impl<const N: usize> Buffer for SmallVec<[u8; N]> {
    #[inline]
    fn clear(&mut self) {
        <SmallVec<[u8; N]>>::clear(self)
    }

    #[inline]
    fn capacity(&self) -> Option<usize> {
        Some(<SmallVec<[u8; N]>>::capacity(self))
    }

    #[inline]
    fn grow(&mut self, length: usize) {
        <SmallVec<[u8; N]>>::resize(self, calculate_new_length_for_grow(self.len(), length), 0u8)
    }
}
