use crate::index::MemIdx;

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
    offset: u64,
    align: Align,
    memory: MemIdx,
}

impl MemArg {
    /// Creates a new [`MemArg`].
    pub const fn new(offset: u64, align: Align, memory: MemIdx) -> Self {
        Self {
            offset,
            align,
            memory,
        }
    }

    /// Gets the offset.
    pub const fn offset(&self) -> u64 {
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

    /// Returns `true` if the `memarg` references a memory other than the default memory (index
    /// `0`).
    ///
    /// See the [multiple memory proposal](https://github.com/WebAssembly/multi-memory) for more
    /// information.
    pub const fn requires_multi_memory(&self) -> bool {
        self.memory.to_u32() != 0
    }

    /// Returns `true` if the `memarg` requires the
    /// [64-bit memory proposal](https://github.com/WebAssembly/memory64).
    pub const fn requires_memory_64(&self) -> bool {
        self.offset > u32::MAX as u64
    }
}

macro_rules! match_instruction_memarg {
    ($memarg:pat) => {
        Self::I32Load($memarg)
            | Self::I64Load($memarg)
            | Self::F32Load($memarg)
            | Self::F64Load($memarg)
            | Self::I32Load8S($memarg)
            | Self::I32Load8U($memarg)
            | Self::I32Load16S($memarg)
            | Self::I32Load16U($memarg)
            | Self::I64Load8S($memarg)
            | Self::I64Load8U($memarg)
            | Self::I64Load16S($memarg)
            | Self::I64Load16U($memarg)
            | Self::I64Load32S($memarg)
            | Self::I64Load32U($memarg)
            | Self::I32Store($memarg)
            | Self::I64Store($memarg)
            | Self::F32Store($memarg)
            | Self::F64Store($memarg)
            | Self::I32Store8($memarg)
            | Self::I32Store16($memarg)
            | Self::I64Store8($memarg)
            | Self::I64Store16($memarg)
            | Self::I64Store32($memarg)
            | Self::V128Load($memarg)
            | Self::V128Load8x8S($memarg)
            | Self::V128Load8x8U($memarg)
            | Self::V128Load16x4S($memarg)
            | Self::V128Load16x4U($memarg)
            | Self::V128Load32x2S($memarg)
            | Self::V128Load32x2U($memarg)
            | Self::V128Load8Splat($memarg)
            | Self::V128Load16Splat($memarg)
            | Self::V128Load32Splat($memarg)
            | Self::V128Load64Splat($memarg)
            | Self::V128Load32Zero($memarg)
            | Self::V128Load64Zero($memarg)
            | Self::V128Store($memarg)
            | Self::V128Load8Lane($memarg, _)
            | Self::V128Load16Lane($memarg, _)
            | Self::V128Load32Lane($memarg, _)
            | Self::V128Load64Lane($memarg, _)
            | Self::V128Store8Lane($memarg, _)
            | Self::V128Store16Lane($memarg, _)
            | Self::V128Store32Lane($memarg, _)
            | Self::V128Store64Lane($memarg, _)
            | Self::MemoryAtomicNotify($memarg)
            | Self::MemoryAtomicWait32($memarg)
            | Self::MemoryAtomicWait64($memarg)
            | Self::I32AtomicLoad($memarg)
            | Self::I64AtomicLoad($memarg)
            | Self::I32AtomicLoad8U($memarg)
            | Self::I32AtomicLoad16U($memarg)
            | Self::I64AtomicLoad8U($memarg)
            | Self::I64AtomicLoad16U($memarg)
            | Self::I64AtomicLoad32U($memarg)
            | Self::I32AtomicStore($memarg)
            | Self::I64AtomicStore($memarg)
            | Self::I32AtomicStore8U($memarg)
            | Self::I32AtomicStore16U($memarg)
            | Self::I64AtomicStore8U($memarg)
            | Self::I64AtomicStore16U($memarg)
            | Self::I64AtomicStore32U($memarg)
            | Self::I32AtomicRmwAdd($memarg)
            | Self::I64AtomicRmwAdd($memarg)
            | Self::I32AtomicRmw8AddU($memarg)
            | Self::I32AtomicRmw16AddU($memarg)
            | Self::I64AtomicRmw8AddU($memarg)
            | Self::I64AtomicRmw16AddU($memarg)
            | Self::I64AtomicRmw32AddU($memarg)
            | Self::I32AtomicRmwSub($memarg)
            | Self::I64AtomicRmwSub($memarg)
            | Self::I32AtomicRmw8SubU($memarg)
            | Self::I32AtomicRmw16SubU($memarg)
            | Self::I64AtomicRmw8SubU($memarg)
            | Self::I64AtomicRmw16SubU($memarg)
            | Self::I64AtomicRmw32SubU($memarg)
            | Self::I32AtomicRmwAnd($memarg)
            | Self::I64AtomicRmwAnd($memarg)
            | Self::I32AtomicRmw8AndU($memarg)
            | Self::I32AtomicRmw16AndU($memarg)
            | Self::I64AtomicRmw8AndU($memarg)
            | Self::I64AtomicRmw16AndU($memarg)
            | Self::I64AtomicRmw32AndU($memarg)
            | Self::I32AtomicRmwOr($memarg)
            | Self::I64AtomicRmwOr($memarg)
            | Self::I32AtomicRmw8OrU($memarg)
            | Self::I32AtomicRmw16OrU($memarg)
            | Self::I64AtomicRmw8OrU($memarg)
            | Self::I64AtomicRmw16OrU($memarg)
            | Self::I64AtomicRmw32OrU($memarg)
            | Self::I32AtomicRmwXor($memarg)
            | Self::I64AtomicRmwXor($memarg)
            | Self::I32AtomicRmw8XorU($memarg)
            | Self::I32AtomicRmw16XorU($memarg)
            | Self::I64AtomicRmw8XorU($memarg)
            | Self::I64AtomicRmw16XorU($memarg)
            | Self::I64AtomicRmw32XorU($memarg)
            | Self::I32AtomicRmwXchg($memarg)
            | Self::I64AtomicRmwXchg($memarg)
            | Self::I32AtomicRmw8XchgU($memarg)
            | Self::I32AtomicRmw16XchgU($memarg)
            | Self::I64AtomicRmw8XchgU($memarg)
            | Self::I64AtomicRmw16XchgU($memarg)
            | Self::I64AtomicRmw32XchgU($memarg)
            | Self::I32AtomicRmwCmpxchg($memarg)
            | Self::I64AtomicRmwCmpxchg($memarg)
            | Self::I32AtomicRmw8CmpxchgU($memarg)
            | Self::I32AtomicRmw16CmpxchgU($memarg)
            | Self::I64AtomicRmw8CmpxchgU($memarg)
            | Self::I64AtomicRmw16CmpxchgU($memarg)
            | Self::I64AtomicRmw32CmpxchgU($memarg)
    };
}

impl<B: crate::bytes::Bytes> crate::instruction_set::Instruction<'_, B> {
    /// Returns `true` if the instruction references memory other than the default memory (index
    /// `0`), which requires the
    /// [multiple memory proposal](https://github.com/WebAssembly/multi-memory).
    pub const fn requires_multi_memory(&self) -> bool {
        match self {
            match_instruction_memarg!(memarg) => memarg.requires_multi_memory(),
            Self::MemorySize(index)
            | Self::MemoryGrow(index)
            | Self::MemoryInit(_, index)
            | Self::MemoryFill(index) => index.to_u32() != 0,
            Self::MemoryCopy {
                destination,
                source,
            } => destination.to_u32() != 0 || source.to_u32() != 0,
            _ => false,
        }
    }

    /// Returns `true` if the instruction would require the
    /// [64-bit memory proposal](https://github.com/WebAssembly/memory64).
    pub const fn requires_memory_64(&self) -> bool {
        match self {
            match_instruction_memarg!(memarg) => memarg.requires_memory_64(),
            _ => false,
        }
    }
}
