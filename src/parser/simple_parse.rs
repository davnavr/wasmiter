/// Provides a [`Parse`](crate::parser::Parse) implementation.
#[derive(Clone)]
pub struct SimpleParse<T> {
    _phantom: core::marker::PhantomData<*const T>,
}

impl<T> Default for SimpleParse<T> {
    #[inline]
    fn default() -> Self {
        Self {
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<T> core::fmt::Debug for SimpleParse<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("SimpleParse").finish()
    }
}
