//! Types and traits for handling allocations during WebAssembly parsing.

mod own_or_ref;
mod string_pool;

pub use own_or_ref::OwnOrRef;
pub use string_pool::StringPool;

#[cfg(feature = "alloc")]
pub use string_pool::FakeStringPool;

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

/// Trait for heap allocation.
pub trait Allocator {
    /// A type for byte buffers.
    type Buf: Buffer;

    /// Allocates a new buffer.
    fn allocate_buffer(&self) -> Self::Buf;

    /// A type for allocated strings.
    type String: AsRef<str> + AsMut<str>;

    /// Allocates a new string.
    fn allocate_string(&self, s: &str) -> Self::String;
}

impl<A: Allocator> Allocator for &A {
    type Buf = A::Buf;

    #[inline]
    fn allocate_buffer(&self) -> Self::Buf {
        A::allocate_buffer(self)
    }

    type String = A::String;

    #[inline]
    fn allocate_string(&self, s: &str) -> Self::String {
        A::allocate_string(self, s)
    }
}

/// An [`Allocator`] implementation that uses Rust's heap allocator.
#[derive(Clone, Copy, Debug, Default)]
#[cfg(feature = "alloc")]
pub struct Global;

#[cfg(feature = "alloc")]
impl Allocator for Global {
    type Buf = alloc::vec::Vec<u8>;

    #[inline]
    fn allocate_buffer(&self) -> Self::Buf {
        Default::default()
    }

    type String = alloc::string::String;

    #[inline]
    fn allocate_string(&self, s: &str) -> Self::String {
        alloc::string::ToString::to_string(s)
    }
}
