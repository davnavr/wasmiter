use crate::{index::TypeIdx, types::ValType};

/// Represents a
/// [`blocktype`](https://webassembly.github.io/spec/core/binary/instructions.html#binary-blocktype),
/// which describes the types of the inputs and results of a
/// [block](https://webassembly.github.io/spec/core/binary/instructions.html#control-instructions).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum BlockType {
    /// Indicates a block has no outputs.
    Empty,
    /// A [`typeidx`](TypeIdx) that describes the inputs and results for this block.
    Index(TypeIdx),
    /// A type describing the single output of a block.
    Inline(ValType),
}

impl From<TypeIdx> for BlockType {
    #[inline]
    fn from(index: TypeIdx) -> Self {
        Self::Index(index)
    }
}

impl From<ValType> for BlockType {
    #[inline]
    fn from(ty: ValType) -> Self {
        Self::Inline(ty)
    }
}
