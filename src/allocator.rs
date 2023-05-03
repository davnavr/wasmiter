//! Types and traits for handling allocations during WebAssembly parsing.

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

/// Trait for a growable array.
pub trait Vector<T>: AsRef<[T]> + core::iter::Extend<T> {
    /// Appends an item to the end of the vector.
    fn push(&mut self, item: T) {
        self.extend(core::iter::once(item))
    }
}

#[cfg(feature = "alloc")]
impl<T> Vector<T> for alloc::vec::Vec<T> {
    fn push(&mut self, item: T) {
        alloc::vec::Vec::push(self, item)
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

    /// A type for growable arrays.
    type Vec<T>: Vector<T>;

    /// Allocates an empty vector.
    fn allocate_vector<T>(&self) -> Self::Vec<T>;

    /// Allocates an empty vector with the given `capacity`.
    #[inline]
    fn allocate_vector_with_capacity<T>(&self, capacity: usize) -> Self::Vec<T> {
        self.allocate_vector()
    }
}

impl<A: Allocator> Allocator for &A {
    type Buf = A::Buf;

    fn allocate_buffer(&self) -> Self::Buf {
        A::allocate_buffer(self)
    }

    type String = A::String;

    fn allocate_string(&self, s: &str) -> Self::String {
        A::allocate_string(self, s)
    }

    type Vec<T> = A::Vec<T>;

    fn allocate_vector<T>(&self) -> Self::Vec<T> {
        A::allocate_vector(&self)
    }

    fn allocate_vector_with_capacity<T>(&self, capacity: usize) -> Self::Vec<T> {
        A::allocate_vector_with_capacity(&self, capacity)
    }
}

/// An [`Allocator`] implementation that uses Rust's heap allocator.
#[derive(Clone, Copy, Debug, Default)]
#[cfg(feature = "alloc")]
pub struct Global;

#[cfg(feature = "alloc")]
impl Allocator for Global {
    type Buf = alloc::vec::Vec<u8>;

    fn allocate_buffer(&self) -> Self::Buf {
        Default::default()
    }

    type String = alloc::string::String;

    fn allocate_string(&self, s: &str) -> Self::String {
        alloc::string::ToString::to_string(s)
    }

    type Vec<T> = alloc::vec::Vec<T>;

    fn allocate_vector<T>(&self) -> Self::Vec<T> {
        Default::default()
    }

    fn allocate_vector_with_capacity<T>(&self, capacity: usize) -> Self::Vec<T> {
        alloc::vec::Vec::with_capacity(capacity)
    }
}
