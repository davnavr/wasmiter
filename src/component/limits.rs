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
    minimum: u32,
    maximum: Option<u32>,
    share: Sharing,
}

impl Limits {
    /// Creates new [`Limits`].
    pub const fn new(minimum: u32, maximum: Option<u32>, share: Sharing) -> Option<Self> {
        match maximum {
            Some(max) if max < minimum => None,
            _ => Some(Self {
                minimum,
                maximum,
                share,
            }),
        }
    }

    /// The minimum.
    #[inline]
    pub const fn minimum(&self) -> u32 {
        self.minimum
    }

    /// The optional minimum.
    #[inline]
    pub const fn maximum(&self) -> Option<u32> {
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
}

/// A
/// [WebAssembly memory type](https://webassembly.github.io/spec/core/binary/types.html#memory-types),
/// with a [`Limits`] value indicating the minimum and maximum number of pages.
pub type MemType = Limits;
