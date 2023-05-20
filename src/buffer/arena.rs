use crate::buffer::Buffer;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

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

/// [`Arena`] implementation that uses the default heap allocator.
#[cfg(feature = "alloc")]
#[derive(Clone, Copy, Debug, Default)]
pub struct GlobalArena;

#[cfg(feature = "alloc")]
impl Arena for GlobalArena {
    type Buf = Vec<u8>;

    #[inline]
    fn allocate_buffer(&self, capacity: usize) -> Self::Buf {
        Vec::with_capacity(capacity)
    }

    type String = alloc::boxed::Box<str>;

    #[inline]
    fn allocate_string(&self, s: &str) -> Self::String {
        s.into()
    }
}
