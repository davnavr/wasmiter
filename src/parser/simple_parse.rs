/// Provides a [`Parse`](crate::parser::Parse) implementation.
#[derive(Clone, Debug)]
pub struct SimpleParse<T> {
    _phantom: core::marker::PhantomData<*const T>,
}

impl<T> Default for SimpleParse<T> {
    #[inline]
    fn default() -> Self {
        Self { _phantom: core::marker::PhantomData }
    }
}
