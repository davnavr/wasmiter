/// Describes the minimum and maximum number of pages in a memory or elements in a table.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Limits {
    minimum: u32,
    maximum: Option<u32>,
}

impl Limits {
    /// Creates new [`Limits`].
    pub const fn new(minimum: u32, maximum: Option<u32>) -> Option<Self> {
        match maximum {
            Some(max) if max < minimum => None,
            _ => Some(Self { minimum, maximum }),
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
}

/// A
/// [WebAssembly memory type](https://webassembly.github.io/spec/core/binary/types.html#memory-types),
/// with a [`Limits`] value indicating the minimum and maximum number of pages.
pub type MemType = Limits;