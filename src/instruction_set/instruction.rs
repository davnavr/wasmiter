use crate::component::{self, BlockType, LabelIdx, LocalIdx};
use crate::parser::{input::Input, Result, ResultExt, SimpleParse, Vector};

macro_rules! instructions {
    ($(
        $(#[$meta:meta])*
        $case:ident$([$arguments:tt])? = $name:literal,
    )*) => {
        /// Represents a
        /// [WebAssembly instruction](https://webassembly.github.io/spec/core/syntax/instructions.html).
        #[derive(Debug)]
        #[non_exhaustive]
        pub enum Instruction<I: Input> {$(
            $(#[$meta])*
            $case $($arguments)?,
        )*}

        impl<I: Input> Instruction<I> {
            /// Gets a string containing the name of the [`Instruction`].
            pub fn name(&self) -> &'static str {
                match self {
                    $(Self::$case { .. } => $name,)*
                }
            }
        }
    };
}

instructions! {
    // Control Instructions

    /// The
    /// [**nop**](https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-instr-control)
    /// instruction does nothing.
    Nop = "nop",
    /// The
    /// [**unreachable**](https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-instr-control)
    /// instruction causes an unconditional
    /// [trap](https://webassembly.github.io/spec/core/intro/overview.html#trap), preventing
    /// any instructions that follow from being executed.
    Unreachable = "unreachable",
    /// The
    /// [**block**](https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-instr-control)
    /// instruction marks the start of a block.
    Block[(BlockType)] = "block",
    /// The
    /// [**loop**](https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-instr-control)
    /// instruction marks the start of a block where branches to the block transfer control flow to
    /// the start of the block.
    Loop[(BlockType)] = "loop",
    /// The
    /// [**if**](https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-instr-control)
    /// instruction marks the start of a block that control is transferred to when a condition is
    /// `true`.
    If[(BlockType)] = "if",
    /// The
    /// [**br**](https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-instr-control)
    /// instruction performs an unconditional branch.
    Br[(LabelIdx)] = "br",
    /// The
    /// [**br_if**](https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-instr-control)
    /// instruction performs a conditional branch.
    BrIf[(LabelIdx)] = "br_if",
    /// The
    /// [**br_table**](https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-instr-control)
    /// instruction performs an indirect branch, with the target being determined by an index into
    /// a table of labels.
    ///
    /// The table of labels is encoded as a [`Vector`] containing **at least one** [`LabelIdx`],
    /// with the last label specifies the default target.
    BrTable[(Vector<I, SimpleParse<LabelIdx>>)] = "br_table",
    /// The
    /// [**return**](https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-instr-control)
    /// instruction transfers control flow back to the calling function.
    Return = "return",
    /// The
    /// [**call**](https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-instr-control)
    /// instruction calls a function.
    Call[(component::FuncIdx)] = "call",
    /// The
    /// [**call_indirect**](https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-instr-control)
    /// instruction calls a function from a `funcref` stored in a table.
    CallIndirect[(component::TypeIdx, component::TableIdx)] = "call_indirect",
    /// The
    /// [**else**](https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-instr-control)
    /// instruction marks the start of the block control flow is transferred to if the condition for
    /// an [**if**](Instruction::If) block is `false`.
    Else = "else",
    /// The
    /// [**end**](https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-instr-control)
    /// instruction marks the end of an
    /// [expression](https://webassembly.github.io/spec/core/syntax/instructions.html#expressions)
    /// or a block.
    End = "end",

    // Variable Instructions

    /// The
    /// [**local.get**](https://webassembly.github.io/spec/core/syntax/instructions.html#variable-instructions)
    /// instruction loads the value of a local variable onto the stack.
    LocalGet[(LocalIdx)] = "local.get",
    /// The
    /// [**local.set**](https://webassembly.github.io/spec/core/syntax/instructions.html#variable-instructions)
    /// instruction pops a value from the stack and stores it into a local variable.
    LocalSet[(LocalIdx)] = "local.set",
    /// The
    /// [**local.set**](https://webassembly.github.io/spec/core/syntax/instructions.html#variable-instructions)
    /// instruction pops a value from the stack and stores it into a local variable, pushing the
    /// previous value onto the stack.
    LocalTee[(LocalIdx)] = "local.tee",
    /// The
    /// [**global.get**](https://webassembly.github.io/spec/core/syntax/instructions.html#variable-instructions)
    /// instruction loads the value of a global variable onto the stack.
    GlobalGet[(component::GlobalIdx)] = "global.get",
    /// The
    /// [**global.set**](https://webassembly.github.io/spec/core/syntax/instructions.html#variable-instructions)
    /// instruction pops a value from the stack and stores it into a global variable.
    GlobalSet[(LocalIdx)] = "globalset",
}

impl<I: Input> Instruction<I> {
    /// Completely parses the [`Instruction`] and any of its required arguments.
    pub fn finish(self) -> Result<()> {
        match self {
            Self::BrTable(indices) => indices.finish().context("branch label table"),
            _ => Ok(()),
        }
    }
}
