use crate::{
    bytes::Bytes,
    custom::name::NameMap,
    index::Index,
    parser::{AscendingOrder, Offset, Result, ResultExt as _, Vector},
};

/// An
/// [*indirect name map*](https://webassembly.github.io/spec/core/appendix/custom.html#name-maps)
/// associates [`NameMap`]s with a priamry index.
///
/// Like a [`NameMap`], each primary index is checked to ensure they are unique and in increasing
/// order.
#[derive(Clone, Copy)]
pub struct IndirectNameMap<K: Index, V: Index, O: Offset, B: Bytes> {
    entries: Vector<O, B>,
    order: AscendingOrder<u32, K>,
    _marker: core::marker::PhantomData<V>,
}

impl<K: Index, V: Index, O: Offset, B: Bytes> From<Vector<O, B>> for IndirectNameMap<K, V, O, B> {
    fn from(entries: Vector<O, B>) -> Self {
        Self {
            entries,
            order: AscendingOrder::new(),
            _marker: core::marker::PhantomData,
        }
    }
}

impl<K: Index, V: Index, O: Offset, B: Bytes> IndirectNameMap<K, V, O, B> {
    /// Parses a [`IndirectNameMap`] starting at the given `offset`.
    pub fn new(offset: O, bytes: B) -> Result<Self> {
        Vector::parse(offset, bytes).map(Self::from)
    }

    /// Gets the remaining number of pairs in the [`IndirectNameMap`].
    pub fn remaining_count(&self) -> u32 {
        self.entries.remaining_count()
    }

    /// Parses the next primary index and [`NameMap`] pair.
    pub fn parse<T, F>(&mut self, f: F) -> Result<Option<T>>
    where
        F: FnOnce(K, &mut NameMap<V, &mut u64, &B>) -> Result<T>,
    {
        self.entries
            .advance_with_index(|i, offset, bytes| {
                let primary_index: K = crate::component::index(offset, bytes)
                    .context("could not parse primary index for pair")?;

                self.order
                    .check(primary_index, i == 0)
                    .context("primary index is not valid")?;

                let mut name_map =
                    NameMap::new(offset, bytes).context("could not parse name map for pair")?;

                let result = f(primary_index, &mut name_map)?;
                name_map.finish()?;
                Result::Ok(result)
            })
            .transpose()
            .context("could not parse entry in indirect name map")
    }

    fn borrowed(&self) -> IndirectNameMap<K, V, u64, &B> {
        IndirectNameMap {
            entries: self.entries.borrowed(),
            order: self.order,
            _marker: core::marker::PhantomData,
        }
    }
}

impl<K: Index, V: Index, O: Offset, B: Bytes> core::fmt::Debug for IndirectNameMap<K, V, O, B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        struct Entry<K: Index, V: Index, B: Bytes> {
            key: K,
            names: NameMap<V, u64, B>,
        }

        impl<K: Index, V: Index, B: Bytes> core::fmt::Debug for Entry<K, V, B> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct("Entry")
                    .field("key", &self.key)
                    .field("names", &self.names)
                    .finish()
            }
        }

        let mut entries = self.borrowed();
        let mut list = f.debug_list();
        loop {
            let result = entries.parse(|key, names| {
                list.entry(&Entry {
                    key,
                    names: names.dereferenced(),
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
