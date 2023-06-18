use crate::parser;

/// Helper struct to ensure indices or numeric IDs are in **ascending** order.
#[derive(Clone, Copy)]
pub(crate) struct AscendingOrder<C: Copy + 'static, I: Copy + 'static = C> {
    previous: C,
    _marker: core::marker::PhantomData<I>,
}

impl<C: Copy + 'static, I: Copy + 'static> AscendingOrder<C, I> {
    pub(crate) fn new() -> Self
    where
        C: From<u8>,
    {
        Self {
            previous: C::from(0u8),
            _marker: core::marker::PhantomData,
        }
    }

    pub(crate) fn check(&mut self, next: I, first: bool) -> parser::Result<I>
    where
        C: core::fmt::Display + Send + Sync + 'static,
        I: core::fmt::Debug + Into<C> + core::cmp::PartialOrd<C> + Send + Sync,
    {
        let previous = self.previous;
        if !first && next <= previous {
            #[inline(never)]
            #[cold]
            fn duplicate_encountered<I: core::fmt::Debug + Send + Sync + 'static>(
                next: I,
            ) -> parser::Error {
                parser::Error::new(parser::ErrorKind::InvalidFormat).with_context(
                    parser::Context::from_closure(move |f| write!(f, "duplicate {next:?}")),
                )
            }

            #[inline(never)]
            #[cold]
            fn out_of_order<C, I>(next: I, previous: C) -> parser::Error
            where
                C: core::fmt::Display + Send + Sync + 'static,
                I: core::fmt::Debug + Send + Sync + 'static,
            {
                parser::Error::new(parser::ErrorKind::InvalidFormat).with_context(
                    parser::Context::from_closure(move |f| {
                        write!(
                            f,
                            "must be in ascending order, {next:?} should come after {previous}"
                        )
                    }),
                )
            }

            Err(if next == previous {
                duplicate_encountered(next)
            } else {
                out_of_order(next, previous)
            })
        } else {
            self.previous = Into::<C>::into(next);
            Ok(next)
        }
    }
}
