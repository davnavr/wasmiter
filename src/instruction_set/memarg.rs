use crate::component::MemIdx;

/// Specifies the alignment for a [`memarg`](MemArg).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[allow(missing_docs)]
pub enum Align {
    None,
    Two,
    Four,
    Eight,
    Sixteen,
}

impl Align {
    /// Creates a new alignment value from an exponent of a power of 2.
    pub const fn new(power: u8) -> Option<Self> {
        Some(match power {
            0 => Self::None,
            1 => Self::Two,
            2 => Self::Four,
            3 => Self::Eight,
            4 => Self::Sixteen,
            _ => return None,
        })
    }

    /// Gets the alignment value, expressed as the exponent of a power of 2.
    ///
    /// For example, a value of 0 means no alignment, a value of 1 means alignment at a 2-byte
    /// boundary, a value of 3 means alignment at a 4-byte boundary, and so on.
    pub const fn to_power(self) -> u8 {
        match self {
            Self::None => 0,
            Self::Two => 1,
            Self::Four => 2,
            Self::Eight => 3,
            Self::Sixteen => 4,
        }
    }
}

/// A WebAssembly
/// [`memarg`](https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions)
/// specifies an address **offset** and expected **alignment** for a memory load or store.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct MemArg {
    offset: u32,
    align: Align,
    memory: MemIdx,
}

impl MemArg {
    /// Creates a new [`MemArg`].
    pub const fn new(offset: u32, align: Align, memory: MemIdx) -> Self {
        Self {
            offset,
            align,
            memory,
        }
    }

    /// Gets the offset.
    pub const fn offset(&self) -> u32 {
        self.offset
    }

    /// Gets the alignment value.
    pub const fn align(&self) -> Align {
        self.align
    }

    /// Gets the memory.
    pub const fn memory(&self) -> MemIdx {
        self.memory
    }

    /// Returns `true` if the `memarg` references a memory other than the default memory `0`.
    ///
    /// See the [multiple memory proposal](https://github.com/WebAssembly/multi-memory) for more
    /// information.
    pub const fn requires_multi_memory(&self) -> bool {
        self.memory.to_u32() == 0
    }
}
