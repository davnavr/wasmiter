/// Helper struct to ensure indices or numeric IDs are in **ascending** order.
#[derive(Clone, Copy)]
pub(crate) struct AscendingOrder<C: Copy, I: Copy> {
    previous: C,
    _marker: core::marker::PhantomData<I>,
}

impl<C: Copy, I: Copy> AscendingOrder<C, I> {
    pub(crate) fn new() -> Self
    where
        C: From<u8>,
    {
        Self {
            previous: C::from(0u8),
            _marker: core::marker::PhantomData,
        }
    }

    pub(crate) fn check(&mut self, index: I, first: bool) -> crate::parser::Result<I>
    where
        C: core::fmt::Display,
        I: core::fmt::Debug + Into<C> + core::cmp::PartialOrd<C>,
    {
        let previous_index = self.previous;
        if !first && index <= previous_index {
            Err(if index == previous_index {
                crate::parser_bad_format!("duplicate index {index:?}")
            } else {
                crate::parser_bad_format!("indices must be in ascending order, index {index:?} should come after {previous_index}")
            })
        } else {
            self.previous = Into::<C>::into(index);
            Ok(index)
        }
    }
}
