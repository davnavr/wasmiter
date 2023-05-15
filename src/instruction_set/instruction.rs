use crate::bytes::Bytes;
use crate::component::{self, BlockType, LabelIdx, LocalIdx, MemIdx, TableIdx};
use crate::instruction_set::MemArg;
use crate::parser::{Result, ResultExt, SimpleParse, Vector};

macro_rules! instructions {
    ($(
        $(#[$meta:meta])*
        $case:ident$([$arguments:tt])? = $name:literal,
    )*) => {
        /// Represents a
        /// [WebAssembly instruction](https://webassembly.github.io/spec/core/syntax/instructions.html).
        #[derive(Debug)]
        #[non_exhaustive]
        pub enum Instruction<'a, B: Bytes> {$(
            $(#[$meta])*
            $case $($arguments)?,
        )*}

        impl<B: Bytes> Instruction<'_, B> {
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
    BrTable[(Vector<&'a mut u64, B, SimpleParse<LabelIdx>>)] = "br_table",
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
    CallIndirect[(component::TypeIdx, TableIdx)] = "call_indirect",
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

    // Reference Instructions

    /// The
    /// [**ref.null**](https://webassembly.github.io/spec/core/syntax/instructions.html#reference-instructions)
    /// instruction produces a `null` value of the specified reference type.
    RefNull[(component::RefType)] = "ref.null",
    /// The
    /// [**ref.is_null**](https://webassembly.github.io/spec/core/syntax/instructions.html#reference-instructions)
    /// instruction checks if an operand is `null`.
    RefIsNull = "ref.is_null",
    /// The
    /// [**ref.func**](https://webassembly.github.io/spec/core/syntax/instructions.html#reference-instructions)
    /// instruction produces a reference to a given function (a `funcref`).
    RefFunc[(component::FuncIdx)] = "ref.func",

    // Parametric Instructions

    /// The
    /// [**drop**](https://webassembly.github.io/spec/core/syntax/instructions.html#parametric-instructions)
    /// instruction discards an operand from the value stack.
    Drop = "drop",
    /// The
    /// [**select**](https://webassembly.github.io/spec/core/syntax/instructions.html#parametric-instructions)
    /// instruction selects one of two operands based on a third condition operand.
    ///
    /// The types specify the type of the operand selected. Future versions of WebAssembly may
    /// allow selecting more than one value at a time, requiring more than one type.
    Select[(Vector<&'a mut u64, B, SimpleParse<component::ValType>>)] = "select",

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
    GlobalSet[(LocalIdx)] = "global.set",

    // Table Instructions

    /// The
    /// [**table.get**](https://webassembly.github.io/spec/core/syntax/instructions.html#table-instructions)
    /// instruction loads an element in the specified table.
    TableGet[(TableIdx)] = "table.get",
    /// The
    /// [**table.set**](https://webassembly.github.io/spec/core/syntax/instructions.html#table-instructions)
    /// instruction stores an element in the specified table.
    TableSet[(TableIdx)] = "table.set",
    /// The
    /// [**table.init**](https://webassembly.github.io/spec/core/syntax/instructions.html#table-instructions)
    /// instruction copies elements from a
    /// [passive element segment](https://webassembly.github.io/spec/core/syntax/modules.html#syntax-elem)
    /// into the specified table.
    TableInit[(component::ElemIdx, TableIdx)] = "table.init",
    /// The
    /// [**elem.drop**](https://webassembly.github.io/spec/core/syntax/instructions.html#table-instructions)
    /// instruction serves as a hint that the given
    /// [element segment](https://webassembly.github.io/spec/core/syntax/modules.html#syntax-elem)
    /// will no longer be used.
    ElemDrop[(component::ElemIdx)] = "elem.drop",
    /// The
    /// [**table.copy**](https://webassembly.github.io/spec/core/syntax/instructions.html#table-instructions)
    /// instruction copies elements from the `source` table into the `destination` table.
    TableCopy[{
        /// The table elements are copied into.
        destination: TableIdx,
        /// The table elements are copied from.
        source: TableIdx
    }] = "table.copy",
    /// The
    /// [**table.grow**](https://webassembly.github.io/spec/core/syntax/instructions.html#table-instructions)
    /// instruction increases the number of elements that can be stored in a table.
    TableGrow[(TableIdx)] = "table.grow",
    /// The
    /// [**table.size**](https://webassembly.github.io/spec/core/syntax/instructions.html#table-instructions)
    /// instruction returns the current number of elements in the table.
    TableSize[(TableIdx)] = "table.size",
    /// The
    /// [**table.fill**](https://webassembly.github.io/spec/core/syntax/instructions.html#table-instructions)
    /// instruction sets all elements in the table to the value specified by an operand.
    TableFill[(TableIdx)] = "table.fill",

    // Memory Instructions

    /// The
    /// [**i32.load**](https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions)
    /// instruction reads a 32-bit integer from memory.
    I32Load[(MemArg)] = "i32.load",
    /// The
    /// [**i64.load**](https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions)
    /// instruction reads a 64-bit integer from memory.
    I64Load[(MemArg)] = "i64.load",
    /// The
    /// [**f32.load**](https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions)
    /// instruction reads a 32-bit IEEE-754 float from memory.
    F32Load[(MemArg)] = "f32.load",
    /// The
    /// [**f64.load**](https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions)
    /// instruction reads a 64-bit IEEE-754 float from memory.
    F64Load[(MemArg)] = "f64.load",

    /// The
    /// [**i32.load8_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions)
    /// instruction reads a byte from memory, and sign-extends it into a 32-bit integer.
    I32Load8S[(MemArg)] = "i32.load8_s",
    /// The
    /// [**i32.load8_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions)
    /// instruction reads a byte from memory, and interprets zero-extends it into a 32-bit integer.
    I32Load8U[(MemArg)] = "i32.load8_u",
    /// The
    /// [**i32.load16_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions)
    /// instruction reads a 16-bit integer from memory, and sign-extends it into a 32-bit integer.
    I32Load16S[(MemArg)] = "i32.load16_s",
    /// The
    /// [**i32.load16_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions)
    /// instruction reads a 16-bit integer from memory, and interprets zero-extends it into a 32-bit integer.
    I32Load16U[(MemArg)] = "i32.load16_u",

    /// The
    /// [**i64.load8_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions)
    /// instruction reads a byte from memory, and sign-extends it into a 64-bit integer.
    I64Load8S[(MemArg)] = "i64.load8_s",
    /// The
    /// [**i64.load8_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions)
    /// instruction reads a byte from memory, and interprets zero-extends it into a 64-bit integer.
    I64Load8U[(MemArg)] = "i64.load8_u",
    /// The
    /// [**i64.load16_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions)
    /// instruction reads a 16-bit integer from memory, and sign-extends it into a 64-bit integer.
    I64Load16S[(MemArg)] = "i64.load16_s",
    /// The
    /// [**i64.load16_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions)
    /// instruction reads a 16-bit integer from memory, and interprets zero-extends it into a 64-bit integer.
    I64Load16U[(MemArg)] = "i64.load16_u",
    /// The
    /// [**i64.load32_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions)
    /// instruction reads a 32-bit integer from memory, and sign-extends it into a 64-bit integer.
    I64Load32S[(MemArg)] = "i64.load32_s",
    /// The
    /// [**i64.load32_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions)
    /// instruction reads a 32-bit integer from memory, and interprets zero-extends it into a 64-bit integer.
    I64Load32U[(MemArg)] = "i64.load32_u",

    /// The
    /// [**i32.load**](https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions)
    /// instruction writes a 32-bit integer into memory.
    I32Store[(MemArg)] = "i32.store",
    /// The
    /// [**i64.load**](https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions)
    /// instruction writes a 64-bit integer into memory.
    I64Store[(MemArg)] = "i64.store",
    /// The
    /// [**f32.load**](https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions)
    /// instruction writes a 32-bit IEEE-754 float into memory.
    F32Store[(MemArg)] = "f32.store",
    /// The
    /// [**f64.load**](https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions)
    /// instruction writes a 64-bit IEEE-754 float into memory.
    F64Store[(MemArg)] = "f64.store",

    /// The
    /// [**i32.store8**](https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions)
    /// instruction writes a byte into memory.
    I32Store8[(MemArg)] = "i32.store8",
    /// The
    /// [**i32.store16**](https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions)
    /// instruction writes a 16-bit integer into memory.
    I32Store16[(MemArg)] = "i32.store16",

    /// The
    /// [**i64.store8**](https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions)
    /// instruction writes a byte into memory.
    I64Store8[(MemArg)] = "i64.store8",
    /// The
    /// [**i64.store16**](https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions)
    /// instruction writes a 16-bit integer into memory.
    I64Store16[(MemArg)] = "i64.store16",
    /// The
    /// [**i64.store32**](https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions)
    /// instruction writes a 32-bit integer into memory.
    I64Store32[(MemArg)] = "i64.store32",

    /// The
    /// [**memory.size**](https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions)
    /// instruction returns the current number of pages allocated for the given memory.
    MemorySize[(MemIdx)] = "memory.size",
    /// The
    /// [**memory.grow**](https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions)
    /// instruction increases the number of pages allocated for the given memory by an amount.
    MemoryGrow[(MemIdx)] = "memory.grow",
    /// The
    /// [**memory.init**](https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions)
    /// instruction copies bytes from a
    /// [passive data segment](https://webassembly.github.io/spec/core/syntax/modules.html#syntax-data)
    /// into the given memory.
    MemoryInit[(component::DataIdx, MemIdx)] = "memory.init",
    /// The
    /// [**data.drop**](https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions)
    /// instruction serves as a hint that the given
    /// [data segment](https://webassembly.github.io/spec/core/syntax/modules.html#syntax-data)
    /// will no longer be used.
    DataDrop[(component::DataIdx)] = "data.drop",
    /// The
    /// [**memory.copy**](https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions)
    /// instruction copies bytes from one memory into another memory.
    MemoryCopy[{
        /// The memory that bytes are copied into.
        destination: MemIdx,
        /// The memory that bytes are copied from.
        source: MemIdx,
    }] = "memory.copy",
    /// The
    /// [**memory.fill**](https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions)
    /// instruction fills a region of memory with byte value.
    MemoryFill[(MemIdx)] = "memory.fill",

    // Numeric Instructions

    /// The
    /// [**i32.const**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    /// instruction returns a 32-bit integer constant.
    I32Const[(i32)] = "i32.const",
    /// The
    /// [**i64.const**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    /// instruction returns a 64-bit integer constant.
    I64Const[(i64)] = "i64.const",
    /// The
    /// [**f32.const**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    /// instruction returns a 32-bit IEEE-754 floating point constant.
    F32Const[(f32)] = "f32.const",
    /// The
    /// [**f64.const**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    /// instruction returns a 64-bit IEEE-754 floating point constant.
    F64Const[(f64)] = "f64.const",

    /// [**i32.eqz**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I32Eqz = "i32.eqz",
    /// [**i32.eq**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I32Eq = "i32.eq",
    /// [**i32.ne**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I32Ne = "i32.ne",
    /// [**i32.lt_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I32LtS = "i32.lt_s",
    /// [**i32.lt_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I32LtU = "i32.lt_u",
    /// [**i32.gt_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I32GtS = "i32.gt_s",
    /// [**i32.gt_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I32GtU = "i32.gt_u",
    /// [**i32.le_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I32LeS = "i32.le_s",
    /// [**i32.le_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I32LeU = "i32.le_u",
    /// [**i32.ge_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I32GeS = "i32.ge_s",
    /// [**i32.ge_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I32GeU = "i32.ge_u",

    /// [**i64.eqz**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I64Eqz = "i64.eqz",
    /// [**i64.eq**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I64Eq = "i64.eq",
    /// [**i64.ne**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I64Ne = "i64.ne",
    /// [**i64.lt_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I64LtS = "i64.lt_s",
    /// [**i64.lt_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I64LtU = "i64.lt_u",
    /// [**i64.gt_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I64GtS = "i64.gt_s",
    /// [**i64.gt_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I64GtU = "i64.gt_u",
    /// [**i64.le_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I64LeS = "i64.le_s",
    /// [**i64.le_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I64LeU = "i64.le_u",
    /// [**i64.ge_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I64GeS = "i64.ge_s",
    /// [**i64.ge_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I64GeU = "i64.ge_u",

    /// [**f32.eq**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    F32Eq = "f32.eq",
    /// [**f32.ne**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    F32Ne = "f32.ne",
    /// [**f32.lt**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    F32Lt = "f32.lt",
    /// [**f32.gt**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    F32Gt = "f32.gt",
    /// [**f32.le_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    F32Le = "f32.le",
    /// [**f32.ge**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    F32Ge = "f32.ge",

    /// [**f64.eq**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    F64Eq = "f64.eq",
    /// [**f64.ne**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    F64Ne = "f64.ne",
    /// [**f64.lt**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    F64Lt = "f64.lt",
    /// [**f64.gt**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    F64Gt = "f64.gt",
    /// [**f64.le_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    F64Le = "f64.le",
    /// [**f64.ge**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    F64Ge = "f64.ge",

    /// [**i32.clz**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I32Clz = "i32.clz",
    /// [**i32.ctz**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I32Ctz = "i32.ctz",
    /// [**i32.popcnt**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I32Popcnt = "i32.popcnt",
    /// [**i32.add**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I32Add = "i32.add",
    /// [**i32.sub**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I32Sub = "i32.sub",
    /// [**i32.mul**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I32Mul = "i32.mul",
    /// [**i32.div_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I32DivS = "i32.div_s",
    /// [**i32.div_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I32DivU = "i32.div_u",
    /// [**i32.rem_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I32RemS = "i32.rem_s",
    /// [**i32.rem_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I32RemU = "i32.rem_u",
    /// [**i32.and**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I32And = "i32.and",
    /// [**i32.or**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I32Or = "i32.or",
    /// [**i32.xor**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I32Xor = "i32.xor",
    /// [**i32.shl**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I32Shl = "i32.shl",
    /// [**i32.shr_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I32ShrS = "i32.shr_s",
    /// [**i32.shr_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I32ShrU = "i32.shr_u",
    /// [**i32.rotl**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I32Rotl = "i32.rotl",
    /// [**i32.rotr**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I32Rotr = "i32.rotr",

    /// [**i64.clz**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I64Clz = "i64.clz",
    /// [**i64.ctz**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I64Ctz = "i64.ctz",
    /// [**i64.popcnt**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I64Popcnt = "i64.popcnt",
    /// [**i64.add**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I64Add = "i64.add",
    /// [**i64.sub**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I64Sub = "i64.sub",
    /// [**i64.mul**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I64Mul = "i64.mul",
    /// [**i64.div_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I64DivS = "i64.div_s",
    /// [**i64.div_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I64DivU = "i64.div_u",
    /// [**i64.rem_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I64RemS = "i64.rem_s",
    /// [**i64.rem_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I64RemU = "i64.rem_u",
    /// [**i64.and**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I64And = "i64.and",
    /// [**i64.or**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I64Or = "i64.or",
    /// [**i64.xor**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I64Xor = "i64.xor",
    /// [**i64.shl**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I64Shl = "i64.shl",
    /// [**i64.shr_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I64ShrS = "i64.shr_s",
    /// [**i64.shr_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I64ShrU = "i64.shr_u",
    /// [**i64.rotl**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I64Rotl = "i64.rotl",
    /// [**i64.rotr**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    I64Rotr = "i64.rotr",

    /// [**f32.abs**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    F32Abs = "f32.abs",
    /// [**f32.neg**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    F32Neg = "f32.neg",
    /// [**f32.ceil**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    F32Ceil = "f32.ceil",
    /// [**f32.floor**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    F32Floor = "f32.floor",
    /// [**f32.trunc**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    F32Trunc = "f32.trunc",
    /// [**f32.nearest**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    F32Nearest = "f32.nearest",
    /// [**f32.sqrt**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    F32Sqrt = "f32.sqrt",
    /// [**f32.add**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    F32Add = "f32.add",
    /// [**f32.sub**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    F32Sub = "f32.sub",
    /// [**f32.mul**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    F32Mul = "f32.mul",
    /// [**f32.div**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    F32Div = "f32.div",
    /// [**f32.min**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    F32Min = "f32.min",
    /// [**f32.max**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    F32Max = "f32.max",
    /// [**f32.copysign**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    F32Copysign = "f32.copysign",

    /// [**f64.abs**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    F64Abs = "f64.abs",
    /// [**f64.neg**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    F64Neg = "f64.neg",
    /// [**f64.ceil**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    F64Ceil = "f64.ceil",
    /// [**f64.floor**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    F64Floor = "f64.floor",
    /// [**f64.trunc**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    F64Trunc = "f64.trunc",
    /// [**f64.nearest**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    F64Nearest = "f64.nearest",
    /// [**f64.sqrt**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    F64Sqrt = "f64.sqrt",
    /// [**f64.add**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    F64Add = "f64.add",
    /// [**f64.sub**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    F64Sub = "f64.sub",
    /// [**f64.mul**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    F64Mul = "f64.mul",
    /// [**f64.div**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    F64Div = "f64.div",
    /// [**f64.min**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    F64Min = "f64.min",
    /// [**f64.max**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    F64Max = "f64.max",
    /// [**f64.copysign**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
    F64Copysign = "f64.copysign",
}

impl<B: Bytes> Instruction<'_, B> {
    /// Completely parses the [`Instruction`] and any of its required arguments.
    pub fn finish(self) -> Result<()> {
        match self {
            Self::BrTable(indices) => {
                indices.finish().context("branch label table")?;
            }
            Self::Select(types) => {
                types.finish()?;
            }
            _ => (),
        }
        Ok(())
    }
}
