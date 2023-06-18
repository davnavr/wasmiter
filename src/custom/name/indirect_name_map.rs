use crate::{
    custom::name::NameMap,
    index::Index,
    input::{BorrowInput, CloneInput, HasInput, Input},
    parser::{AscendingOrder, Offset, Result, ResultExt as _, Vector},
};

/// An
/// [*indirect name map*](https://webassembly.github.io/spec/core/appendix/custom.html#name-maps)
/// associates [`NameMap`]s with a priamry index.
///
/// Like a [`NameMap`], each primary index is checked to ensure they are unique and in increasing
/// order.
#[derive(Clone, Copy)]
pub struct IndirectNameMap<K: Index, V: Index, O: Offset, I: Input> {
    entries: Vector<O, I>,
    order: AscendingOrder<u32, K>,
    _marker: core::marker::PhantomData<V>,
}

impl<K: Index, V: Index, O: Offset, I: Input> From<Vector<O, I>> for IndirectNameMap<K, V, O, I> {
    fn from(entries: Vector<O, I>) -> Self {
        Self {
            entries,
            order: AscendingOrder::new(),
            _marker: core::marker::PhantomData,
        }
    }
}

impl<K: Index, V: Index, O: Offset, I: Input> IndirectNameMap<K, V, O, I> {
    /// Parses a [`IndirectNameMap`] starting at the given `offset`.
    pub fn new(offset: O, input: I) -> Result<Self> {
        Vector::parse(offset, input).map(Self::from)
    }

    /// Gets the remaining number of pairs in the [`IndirectNameMap`].
    pub fn remaining_count(&self) -> u32 {
        self.entries.remaining_count()
    }

    /// Parses the next primary index and [`NameMap`] pair.
    pub fn parse<T, F>(&mut self, f: F) -> Result<Option<T>>
    where
        F: FnOnce(K, &mut NameMap<V, &mut u64, &I>) -> Result<T>,
    {
        self.entries
            .advance_with_index(|i, offset, input| {
                let primary_index: K = crate::component::index(offset, input)
                    .context("could not parse primary index for pair")?;

                self.order
                    .check(primary_index, i == 0)
                    .context("primary index is not valid")?;

                let mut name_map =
                    NameMap::new(offset, input).context("could not parse name map for pair")?;

                let result = f(primary_index, &mut name_map)?;
                name_map.finish()?;
                Result::Ok(result)
            })
            .transpose()
            .context("could not parse entry in indirect name map")
    }
}

impl<K: Index, V: Index, O: Offset, I: Input> HasInput<I> for IndirectNameMap<K, V, O, I> {
    #[inline]
    fn input(&self) -> &I {
        self.entries.input()
    }
}

impl<'a, K: Index, V: Index, O: Offset, I: Input + 'a> BorrowInput<'a, I>
    for IndirectNameMap<K, V, O, I>
{
    type Borrowed = IndirectNameMap<K, V, u64, &'a I>;

    #[inline]
    fn borrow_input(&'a self) -> Self::Borrowed {
        IndirectNameMap {
            entries: self.entries.borrow_input(),
            order: self.order,
            _marker: core::marker::PhantomData,
        }
    }
}

impl<K: Index, V: Index, O: Offset, I: Input> core::fmt::Debug for IndirectNameMap<K, V, O, I> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        struct Entry<K: Index, V: Index, I: Input> {
            key: K,
            names: NameMap<V, u64, I>,
        }

        impl<K: Index, V: Index, I: Input> core::fmt::Debug for Entry<K, V, I> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct("Entry")
                    .field("key", &self.key)
                    .field("names", &self.names)
                    .finish()
            }
        }

        let mut entries = self.borrow_input();
        let mut list = f.debug_list();
        loop {
            let result = entries.parse(|key, names| {
                list.entry(&Entry {
                    key,
                    names: names.clone_input(),
                });
                Result::Ok(())
            });

            if let Err(e) = result.as_ref() {
                list.entry(&core::result::Result::<(), _>::Err(e));
            }

            if matches!(result, Ok(None) | Err(_)) {
                break;
            }
        }

        list.finish()
    }
}
