use crate::{bytes::Bytes, custom::name::NameAssoc, index::Index};

/// A [*name map*](https://webassembly.github.io/spec/core/appendix/custom.html#name-maps)
/// is a sequence of [`NameAssoc`] entries, which associate indices with names.
///
/// Each index is checked in order to ensure they are unique and in increasing order.
#[derive(Clone, Copy)]
pub struct NameMap<I: Index, B: Bytes> {
    next: I,
    //entries: Vector,
    entries: core::marker::PhantomData<B>,
}

impl<I: Index, B: Bytes> Iterator for NameMap<I, B> {
    type Item = NameAssoc<I, B>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        todo!()
    }
}

impl<I: Index, B: Bytes> core::fmt::Debug for NameMap<I, B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        //f.debug_map().entries(entries)
        todo!()
    }
}
