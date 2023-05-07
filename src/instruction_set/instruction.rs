use crate::component::BlockType;

/// Represents a
/// [WebAssembly instruction](https://webassembly.github.io/spec/core/syntax/instructions.html).
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum Instruction {
    /// The
    /// [**nop**](https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-instr-control)
    /// instruction does nothing.
    Nop,
    /// The
    /// [**unreachable**](https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-instr-control)
    /// instruction causes an unconditional
    /// [trap](https://webassembly.github.io/spec/core/intro/overview.html#trap), preventing
    /// any instructions that follow from being executed.
    Unreachable,
    /// The
    /// [**end**](https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-instr-control)
    /// instruction marks the end of an
    /// [expression](https://webassembly.github.io/spec/core/syntax/instructions.html#expressions)
    /// or a block.
    End,
    /// The
    /// [**block**](https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-instr-control)
    /// marks the start of a block.
    Block(BlockType),
}
