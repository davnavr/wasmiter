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

/// Trait for a growable array.
pub trait Vector<T>: AsRef<[T]> + core::iter::Extend<T> {
    /// Removes all items from the vector.
    fn clear(&mut self);

    /// Appends an item to the end of the vector.
    fn push(&mut self, item: T) {
        self.extend(core::iter::once(item))
    }

    /// Reserves space for `additional` instances of `T`, possibly reserving extra space.
    fn reserve(&mut self, additional: usize) {
        let _ = additional;
    }

    /// Reserves the minimum amount of space for `additional` instances of `T`.
    fn reserve_exact(&mut self, additional: usize) {
        self.reserve(additional)
    }
}

#[cfg(feature = "alloc")]
impl<T> Vector<T> for alloc::vec::Vec<T> {
    fn clear(&mut self) {
        alloc::vec::Vec::clear(self)
    }

    fn push(&mut self, item: T) {
        alloc::vec::Vec::push(self, item)
    }

    fn reserve(&mut self, additional: usize) {
        alloc::vec::Vec::reserve(self, additional)
    }

    fn reserve_exact(&mut self, additional: usize) {
        alloc::vec::Vec::reserve_exact(self, additional)
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
        let _ = capacity;
        self.allocate_vector()
    }

    /// Allocates a vector from the given slice.
    #[inline]
    fn allocate_vector_from_slice<T: Clone>(&self, items: &[T]) -> Self::Vec<T> {
        let mut vec = self.allocate_vector_with_capacity(items.len());
        vec.extend(items.iter().cloned());
        vec
    }
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

    type Vec<T> = A::Vec<T>;

    #[inline]
    fn allocate_vector<T>(&self) -> Self::Vec<T> {
        A::allocate_vector(self)
    }

    #[inline]
    fn allocate_vector_with_capacity<T>(&self, capacity: usize) -> Self::Vec<T> {
        A::allocate_vector_with_capacity(self, capacity)
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

    type Vec<T> = alloc::vec::Vec<T>;

    #[inline]
    fn allocate_vector<T>(&self) -> Self::Vec<T> {
        Default::default()
    }

    #[inline]
    fn allocate_vector_with_capacity<T>(&self, capacity: usize) -> Self::Vec<T> {
        alloc::vec::Vec::with_capacity(capacity)
    }

    #[inline]
    fn allocate_vector_from_slice<T: Clone>(&self, items: &[T]) -> Self::Vec<T> {
        alloc::vec::Vec::from(items)
    }
}
