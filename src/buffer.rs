//! Types and traits for temporary allocations used during WebAssembly parsing.

mod arena;

pub use arena::Arena;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg(feature = "alloc")]
pub use arena::GlobalArena;

/// Trait for byte buffers.
pub trait Buffer: AsRef<[u8]> + AsMut<[u8]> {
    /// Sets the buffer's length to zero.
    fn clear(&mut self);

    /// Gets the number of bytes that this buffer can contain before requiring reallocation.
    fn capacity(&self) -> Option<usize> {
        None
    }

    /// Increases the buffer's length by the given amount.
    fn grow(&mut self, length: usize);
}

#[inline]
fn calculate_new_length_for_grow(current: usize, additional: usize) -> usize {
    current.checked_add(additional).unwrap_or(usize::MAX)
}

#[cfg(feature = "alloc")]
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

#[cfg(feature = "alloc")]
impl<const N: usize> Buffer for smallvec::SmallVec<[u8; N]> {
    #[inline]
    fn clear(&mut self) {
        <smallvec::SmallVec<[u8; N]>>::clear(self)
    }

    #[inline]
    fn capacity(&self) -> Option<usize> {
        Some(<smallvec::SmallVec<[u8; N]>>::capacity(self))
    }

    #[inline]
    fn grow(&mut self, length: usize) {
        <smallvec::SmallVec<[u8; N]>>::resize(
            self,
            calculate_new_length_for_grow(self.len(), length),
            0u8,
        )
    }
}
