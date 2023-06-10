/// Helper struct to ensure indices or numeric IDs are in **ascending** order.
#[derive(Clone, Copy)]
pub(crate) struct AscendingOrder<C: Copy, I: Copy = C> {
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

    pub(crate) fn check(&mut self, next: I, first: bool) -> crate::parser::Result<I>
    where
        C: core::fmt::Display,
        I: core::fmt::Debug + Into<C> + core::cmp::PartialOrd<C>,
    {
        let previous = self.previous;
        if !first && next <= previous {
            Err(if next == previous {
                crate::parser_bad_format!("duplicate {next:?}")
            } else {
                crate::parser_bad_format!("must be in ascending order, {next:?} should come after {previous}")
            })
        } else {
            self.previous = Into::<C>::into(next);
            Ok(next)
        }
    }
}
