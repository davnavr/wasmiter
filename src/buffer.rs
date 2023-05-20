//! Types and traits for byte buffers used during WebAssembly parsing.

/// Trait for byte buffers.
pub trait Buffer: AsRef<[u8]> + AsMut<[u8]> {
    /// Sets the buffer's length to zero.
    fn clear(&mut self);

    /// Increases the buffer's length by the given amount.
    fn grow(&mut self, length: usize);
}

#[cfg(feature = "alloc")]
impl Buffer for alloc::vec::Vec<u8> {
    fn clear(&mut self) {
        alloc::vec::Vec::clear(self)
    }

    fn grow(&mut self, length: usize) {
        self.resize(self.len().checked_add(length).unwrap_or(usize::MAX), 0u8)
    }
}
