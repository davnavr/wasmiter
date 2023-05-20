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

#[cfg(feature = "alloc")]
impl Buffer for Vec<u8> {
    #[inline]
    fn clear(&mut self) {
        Vec::clear(self)
    }

    #[inline]
    fn capacity(&self) -> Option<usize> {
        Some(Vec::capacity(self))
    }

    #[inline]
    fn grow(&mut self, length: usize) {
        self.resize(self.len().checked_add(length).unwrap_or(usize::MAX), 0u8)
    }
}
