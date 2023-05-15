use core::num::NonZeroU8;

/// A WebAssembly
/// [`memarg`](https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions)
/// specifies an address **offset** and expected **alignment** for a memory load or store.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct MemArg {
    offset: u32,
    align: NonZeroU8,
}

impl MemArg {
    /// Creates a new [`MemArg`].
    pub const fn new(offset: u32, align: NonZeroU8) -> Self {
        Self { offset, align }
    }

    /// Gets the offset.
    pub const fn offset(&self) -> u32 {
        self.offset
    }

    /// Gets the alignment value, expressed as the exponent of a power of 2.
    ///
    /// For example, a value of 0 means no alignment, a value of 1 means alignment at a 2-byte
    /// boundary, a value of 3 means alignment at a 4-byte boundary, and so on.
    pub const fn align(&self) -> NonZeroU8 {
        self.align
    }
}
