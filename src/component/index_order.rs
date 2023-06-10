use crate::index::Index;

/// Helper struct to ensure indices are in **ascending** order.
#[derive(Clone, Copy)]
pub(crate) struct IndexOrder<I: Index> {
    previous: u32,
    _marker: core::marker::PhantomData<I>,
}

impl<I: Index> IndexOrder<I> {
    pub(crate) fn new() -> Self {
        Self {
            previous: 0,
            _marker: core::marker::PhantomData,
        }
    }

    pub(crate) fn check(&mut self, index: I, first: bool) -> crate::parser::Result<I> {
        let previous_index = self.previous;
        if !first && index <= previous_index {
            Err(if index == previous_index {
                crate::parser_bad_format!("duplicate index {index:?}")
            } else {
                crate::parser_bad_format!("indices must be in ascending order, index {index:?} should come after {previous_index}")
            })
        } else {
            self.previous = Into::<u32>::into(index);
            Ok(index)
        }
    }
}
