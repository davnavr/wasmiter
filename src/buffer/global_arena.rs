use alloc::vec::Vec;

/// [`Arena`](crate::buffer::Arena) implementation that uses the default heap allocator.
#[derive(Clone, Copy, Debug, Default)]
pub struct GlobalArena;

impl crate::buffer::Arena for GlobalArena {
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
