use crate::buffer::Buffer;

/// Trait for arena allocation.
pub trait Arena {
    /// Type for byte buffers.
    type Buf: Buffer;

    /// Allocates an empty buffer with the specified `capacity`.
    fn allocate_buffer(&self, capacity: usize) -> Self::Buf;

    /// Type for buffers containing UTF-8 strings.
    type String: AsRef<str>;

    /// Allocates a new string with the given content.
    fn allocate_string(&self, s: &str) -> Self::String;
}

impl<A: Arena> Arena for &A {
    type Buf = A::Buf;

    #[inline]
    fn allocate_buffer(&self, capacity: usize) -> A::Buf {
        A::allocate_buffer(self, capacity)
    }

    type String = A::String;

    #[inline]
    fn allocate_string(&self, s: &str) -> A::String {
        A::allocate_string(self, s)
    }
}
