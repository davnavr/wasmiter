/// Indicates the size of indices into a linear memory or table.
///
/// See the [WebAssembly 64-bit memory proposal](https://github.com/WebAssembly/memory64) for more
/// information.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum IdxType {
    /// The memory or table is indexed by a 32-bit integer, as it was in the WebAssembly 1.0 release.
    #[default]
    I32,
    /// The memory (or table, which might be supported in the future), is indexed by a 64-bit integer.
    ///
    /// This requires the [`memory64` proposal](https://github.com/WebAssembly/memory64).
    I64,
}

/// Indicates whether a linear memory or table is shared, the semantics of which is described in
/// the [WebAssembly threads proposal](https://github.com/webassembly/threads).
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum Sharing {
    /// The linear memory or table can be used in multiple agents.
    Shared,
    /// The linear memory or table can only be used in a single agent.
    #[default]
    Unshared,
}

/// Describes the minimum and maximum number of pages in a memory or elements in a table.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Limits {
    minimum: u64,
    maximum: Option<u64>,
    share: Sharing,
    index_type: IdxType,
}

impl Limits {
    /// Creates new [`Limits`].
    pub const fn new(
        minimum: u64,
        maximum: Option<u64>,
        share: Sharing,
        index_type: IdxType,
    ) -> Option<Self> {
        match maximum {
            Some(max) if max < minimum => None,
            _ => Some(Self {
                minimum,
                maximum,
                share,
                index_type,
            }),
        }
    }

    /// The minimum.
    #[inline]
    pub const fn minimum(&self) -> u64 {
        self.minimum
    }

    /// The optional minimum.
    #[inline]
    pub const fn maximum(&self) -> Option<u64> {
        self.maximum
    }

    /// Whether or not the linear memory or table is shared.
    ///
    /// See the [WebAssembly threads proposal](https://github.com/webassembly/threads) for more
    /// information.
    #[inline]
    pub const fn share(&self) -> Sharing {
        self.share
    }

    /// The integer type used to index into the linear memory or table.
    #[inline]
    pub const fn index_type(&self) -> IdxType {
        self.index_type
    }

    /// Returns `true` if the [`Limits`] would require the
    /// [64-bit memory proposal](https://github.com/WebAssembly/memory64).
    pub fn requires_memory64(&self) -> bool {
        self.minimum > (u32::MAX as u64)
            || matches!(self.index_type, IdxType::I64)
            || matches!(self.maximum, Some(max) if max > (u32::MAX as u64))
    }
}

/// A
/// [WebAssembly memory type](https://webassembly.github.io/spec/core/binary/types.html#memory-types),
/// with a [`Limits`] value indicating the minimum and maximum number of pages.
pub type MemType = Limits;
