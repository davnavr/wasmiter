//! Types and traits for temporary allocations used during WebAssembly parsing.

mod arena;

pub use arena::Arena;

cfg_if::cfg_if! {
    if #[cfg(feature = "alloc")] {
        mod global_arena;
        mod buffer_vec;

        pub use global_arena::GlobalArena;
    }
}

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
