use crate::{
    bytes::Bytes,
    component,
    index::{self, FuncIdx, LabelIdx, LocalIdx, MemIdx, TableIdx},
    instruction_set::MemArg,
    parser::{Result, ResultExt},
    types::{self, BlockType},
};

/// A WebAssembly
/// [`laneidx`](https://webassembly.github.io/spec/core/binary/instructions.html#vector-instructions)
/// refers to a lane within a 128-bit vector.
pub type LaneIdx = u8;

macro_rules! instruction_debug_impl {
    ($f:ident, $e:expr, $case:ident) => {
        if let Self::$case = $e {
            return $f.debug_tuple(stringify!($case)).finish();
        }
    };
    ($f:ident, $e:expr, $case:ident($(#[$a_meta:meta])* $a:ty)) => {
        if let Self::$case(a) = $e {
            return $f.debug_tuple(stringify!($case)).field(&a).finish();
        }
    };
    ($f:ident, $e:expr, $case:ident($(#[$a_meta:meta])* $a:ty, $(#[$b_meta:meta])* $b:ty)) => {
        if let Self::$case(a, b) = $e {
            return $f.debug_tuple(stringify!($case)).field(&a).field(&b).finish();
        }
    };
    ($f:ident, $e:expr, $case:ident { $($(#[$field_meta:meta])* $field_name:ident: $field_ty:ty $(,)?)+ }) => {
        if let Self::$case { $($field_name,)+ } = $e {
            return $f.debug_struct(stringify!($case))
                $(.field(stringify!($field_name), &$field_name))+
                .finish();
        }
    };
}

macro_rules! instructions {
    ($(
        $(#[$group_meta:meta])*
        $group:ident {$(
            $(#[$meta:meta])*
            $case:ident$([$arguments:tt])? = $name:literal,
        )+}
    )*) => {
        /// Represents a
        /// [WebAssembly instruction](https://webassembly.github.io/spec/core/syntax/instructions.html).
        #[non_exhaustive]
        pub enum Instruction<'a, B: Bytes> {$($(
            $(#[$meta])*
            $case $($arguments)?,
        )*)*}

        impl<B: Bytes> Instruction<'_, B> {
            /// Gets a string containing the name of the [`Instruction`].
            pub const fn name(&self) -> &'static str {
                match self {
                    $($(Self::$case { .. } => $name,)+)*
                }
            }

            $(
                $(#[$group_meta])*
                pub const fn $group(&self) -> bool {
                    match self {
                        $(| Self::$case { .. })+ => true,
                        _ => false,
                    }
                }
            )*
        }

        impl<B: Bytes> core::fmt::Debug for Instruction<'_, B> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                $($(
                    instruction_debug_impl!(f, self, $case $($arguments)?);
                )+)*

                unreachable!()
            }
        }
    };
}

instructions! {
    /// Returns `true` if the [`Instruction`] was introduced as part of the 1.0 release of
    /// WebAssembly in 2017.
    ///
    /// Note that the following proposals change the behavior and rules for some instructions that
    /// were introduced in version 1.0:
    /// - The [multi-value proposal](https://github.com/WebAssembly/multi-value), which allows
    /// additional inputs to and outputs from blocks.
    /// - The [reference types proposal](https://github.com/WebAssembly/reference-types), which
    /// allows additional forms of the [`select`](Instruction::Select) and
    /// [`call_indirect`](Instruction::CallIndirect) instructions along with the introduction of
    /// the `funcref` and `externref` types.
    ///
    /// Users that support only certain proposals must check themselves that instructions
    /// introduced in 1.0 do not make use of unsupported proposals.
    is_from_mvp {
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
        /// instruction marks the start of a block where branches to the block transfer control
        /// flow to the start of the block.
        Loop[(BlockType)] = "loop",
        /// The
        /// [**if**](https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-instr-control)
        /// instruction marks the start of a block that control is transferred to when a condition
        /// is `true`.
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
        /// instruction performs an indirect branch, with the target being determined by an index
        /// into a table of labels.
        ///
        /// The table of labels is encoded as a [`Vector`] containing **at least one**
        /// [`LabelIdx`], with the last label specifies the default target.
        BrTable[(component::IndexVector<LabelIdx, &'a mut u64, B>)] = "br_table",
        /// The
        /// [**return**](https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-instr-control)
        /// instruction transfers control flow back to the calling function.
        Return = "return",
        /// The
        /// [**call**](https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-instr-control)
        /// instruction calls a function.
        Call[(FuncIdx)] = "call",
        /// The
        /// [**call_indirect**](https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-instr-control)
        /// instruction calls a function from a `funcref` stored in a table.
        CallIndirect[(index::TypeIdx, TableIdx)] = "call_indirect",
        /// The
        /// [**else**](https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-instr-control)
        /// instruction marks the start of the block control flow is transferred to if the
        /// condition for an [**if**](Instruction::If) block is `false`.
        Else = "else",
        /// The
        /// [**end**](https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-instr-control)
        /// instruction marks the end of an
        /// [expression](https://webassembly.github.io/spec/core/syntax/instructions.html#expressions)
        /// or a block.
        End = "end",

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
        Select[(component::ResultType<&'a mut u64, B>)] = "select",

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
        /// instruction pops a value from the stack and stores it into a local variable, pushing
        /// the previous value onto the stack.
        LocalTee[(LocalIdx)] = "local.tee",
        /// The
        /// [**global.get**](https://webassembly.github.io/spec/core/syntax/instructions.html#variable-instructions)
        /// instruction loads the value of a global variable onto the stack.
        GlobalGet[(index::GlobalIdx)] = "global.get",
        /// The
        /// [**global.set**](https://webassembly.github.io/spec/core/syntax/instructions.html#variable-instructions)
        /// instruction pops a value from the stack and stores it into a global variable.
        GlobalSet[(index::GlobalIdx)] = "global.set",

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
        /// instruction reads a byte from memory, and interprets zero-extends it into a 32-bit
        /// integer.
        I32Load8U[(MemArg)] = "i32.load8_u",
        /// The
        /// [**i32.load16_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions)
        /// instruction reads a 16-bit integer from memory, and sign-extends it into a 32-bit
        /// integer.
        I32Load16S[(MemArg)] = "i32.load16_s",
        /// The
        /// [**i32.load16_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions)
        /// instruction reads a 16-bit integer from memory, and interprets zero-extends it into a
        /// 32-bit integer.
        I32Load16U[(MemArg)] = "i32.load16_u",

        /// The
        /// [**i64.load8_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions)
        /// instruction reads a byte from memory, and sign-extends it into a 64-bit integer.
        I64Load8S[(MemArg)] = "i64.load8_s",
        /// The
        /// [**i64.load8_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions)
        /// instruction reads a byte from memory, and interprets zero-extends it into a 64-bit
        /// integer.
        I64Load8U[(MemArg)] = "i64.load8_u",
        /// The
        /// [**i64.load16_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions)
        /// instruction reads a 16-bit integer from memory, and sign-extends it into a 64-bit
        /// integer.
        I64Load16S[(MemArg)] = "i64.load16_s",
        /// The
        /// [**i64.load16_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions)
        /// instruction reads a 16-bit integer from memory, and interprets zero-extends it into a
        /// 64-bit integer.
        I64Load16U[(MemArg)] = "i64.load16_u",
        /// The
        /// [**i64.load32_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions)
        /// instruction reads a 32-bit integer from memory, and sign-extends it into a 64-bit
        /// integer.
        I64Load32S[(MemArg)] = "i64.load32_s",
        /// The
        /// [**i64.load32_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions)
        /// instruction reads a 32-bit integer from memory, and interprets zero-extends it into a
        /// 64-bit integer.
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

        /// [**i32.wrap_i64**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
        I32WrapI64 = "i32.wrap_i64",
        /// [**i32.trunc_f32_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
        I32TruncF32S = "i32.trunc_f32_s",
        /// [**i32.trunc_f32_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
        I32TruncF32U = "i32.trunc_f64_u",
        /// [**i32.trunc_f64_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
        I32TruncF64S = "i32.trunc_f64_s",
        /// [**i32.trunc_f64_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
        I32TruncF64U = "i32.trunc_f64_u",
        /// [**i64.extend_i32_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
        I64ExtendI32S = "i64.extend_i32_s",
        /// [**i64.extend_i32_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
        I64ExtendI32U = "i64.extend_i32_u",
        /// [**i64.trunc_f32_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
        I64TruncF32S = "i64.trunc_f32_s",
        /// [**i64.trunc_f32_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
        I64TruncF32U = "i64.trunc_f64_u",
        /// [**i64.trunc_f64_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
        I64TruncF64S = "i64.trunc_f64_s",
        /// [**i64.trunc_f64_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
        I64TruncF64U = "i64.trunc_f64_u",
        /// [**f32.convert_i32_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
        F32ConvertI32S = "f32.convert_i32_s",
        /// [**f32.convert_i32_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
        F32ConvertI32U = "f32.convert_i32_u",
        /// [**f32.convert_i64_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
        F32ConvertI64S = "f32.convert_i64_s",
        /// [**f32.convert_i64_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
        F32ConvertI64U = "f32.convert_i64_u",
        /// [**f32.demote_f64**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
        F32DemoteF64 = "f32.demote_f64",
        /// [**f64.convert_i32_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
        F64ConvertI32S = "f64.convert_i32_s",
        /// [**f64.convert_i32_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
        F64ConvertI32U = "f64.convert_i32_u",
        /// [**f64.convert_i64_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
        F64ConvertI64S = "f64.convert_i64_s",
        /// [**f64.convert_i64_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
        F64ConvertI64U = "f64.convert_i64_u",
        /// [**f64.promote_f32**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
        F64PromoteF32 = "f64.promote_f32",
        /// [**i32.reinterpret_f32**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
        I32ReinterpretF32 = "i32.reinterpret_f32",
        /// [**i64.reinterpret_f64**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
        I64ReinterpretF64 = "i64.reinterpret_f64",
        /// [**f32.reinterpret_i32**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
        F32ReinterpretI32 = "f32.reinterpret_i32",
        /// [**f64.reinterpret_i64**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
        F64ReinterpretI64 = "f64.reinterpret_i64",
    }

    /// Returns `true` if the [`Instruction`] was introduced as part of the
    /// [non-trapping float-to-int conversions proposal](https://github.com/WebAssembly/nontrapping-float-to-int-conversions).
    is_from_non_trapping_float_to_int_conversions {
        /// [**i32.trunc_sat_f32_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
        I32TruncSatF32S = "i32.trunc_sat_f32_s",
        /// [**i32.trunc_sat_f32_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
        I32TruncSatF32U = "i32.trunc_sat_f32_u",
        /// [**i32.trunc_sat_f64_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
        I32TruncSatF64S = "i32.trunc_sat_f64_s",
        /// [**i32.trunc_sat_f64_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
        I32TruncSatF64U = "i32.trunc_sat_f64_u",
        /// [**i64.trunc_sat_f32_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
        I64TruncSatF32S = "i64.trunc_sat_f32_s",
        /// [**i64.trunc_sat_f32_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
        I64TruncSatF32U = "i64.trunc_sat_f32_u",
        /// [**i64.trunc_sat_f64_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
        I64TruncSatF64S = "i64.trunc_sat_f64_s",
        /// [**i64.trunc_sat_f64_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
        I64TruncSatF64U = "i64.trunc_sat_f64_u",
    }

    /// Returns `true` if the [`Instruction`] was introduced as part of the
    /// [sign-extension operators proposal](https://github.com/WebAssembly/sign-extension-ops).
    is_from_sign_extension_operators {
        /// [**i32.extend8_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
        I32Extend8S = "i32.extend8_s",
        /// [**i32.extend16_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
        I32Extend16S = "i32.extend16_s",
        /// [**i64.extend8_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
        I64Extend8S = "i64.extend8_s",
        /// [**i64.extend16_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
        I64Extend16S = "i64.extend16_s",
        /// [**i64.extend32_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#numeric-instructions)
        I64Extend32S = "i64.extend32_s",
    }

    /// Returns `true` if the [`Instruction`] was introduced as part of the
    /// [reference types proposal](https://github.com/WebAssembly/reference-types).
    ///
    /// Note that the `table.fill` and `table.grow` are excluded, as they were also (concurrently?)
    /// introduced in the [bulk memory operations proposal](Instruction::is_from_bulk_memory_operations).
    is_from_reference_types {
        // Reference Instructions

        /// The
        /// [**ref.null**](https://webassembly.github.io/spec/core/syntax/instructions.html#reference-instructions)
        /// instruction produces a `null` value of the specified reference type.
        RefNull[(types::RefType)] = "ref.null",
        /// The
        /// [**ref.is_null**](https://webassembly.github.io/spec/core/syntax/instructions.html#reference-instructions)
        /// instruction checks if an operand is `null`.
        RefIsNull = "ref.is_null",
        /// The
        /// [**ref.func**](https://webassembly.github.io/spec/core/syntax/instructions.html#reference-instructions)
        /// instruction produces a reference to a given function (a `funcref`).
        RefFunc[(FuncIdx)] = "ref.func",

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
        /// [**table.size**](https://webassembly.github.io/spec/core/syntax/instructions.html#table-instructions)
        /// instruction returns the current number of elements in the table.
        TableSize[(TableIdx)] = "table.size",
        /// The
        /// [**table.grow**](https://webassembly.github.io/spec/core/syntax/instructions.html#table-instructions)
        /// instruction increases the number of elements that can be stored in a table.
        TableGrow[(TableIdx)] = "table.grow",
        /// The
        /// [**table.fill**](https://webassembly.github.io/spec/core/syntax/instructions.html#table-instructions)
        /// instruction sets all elements in the table to the value specified by an operand.
        TableFill[(TableIdx)] = "table.fill",
    }

    /// Returns `true` if the [`Instruction`] was introduced as part of the
    /// [bulk memory operations proposal](https://github.com/WebAssembly/bulk-memory-operations).
    is_from_bulk_memory_operations {
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
        /// [**table.copy**](https://webassembly.github.io/spec/core/syntax/instructions.html#table-instructions)
        /// instruction copies elements from the `source` table into the `destination` table.
        TableCopy[{
            /// The table elements are copied into.
            destination: TableIdx,
            /// The table elements are copied from.
            source: TableIdx
        }] = "table.copy",

        /// The
        /// [**memory.fill**](https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions)
        /// instruction fills a region of memory with byte value.
        MemoryFill[(MemIdx)] = "memory.fill",

        /// The
        /// [**memory.init**](https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions)
        /// instruction copies bytes from a
        /// [passive data segment](https://webassembly.github.io/spec/core/syntax/modules.html#syntax-data)
        /// into the given memory.
        MemoryInit[(index::DataIdx, MemIdx)] = "memory.init",
        /// The
        /// [**table.init**](https://webassembly.github.io/spec/core/syntax/instructions.html#table-instructions)
        /// instruction copies elements from a
        /// [passive element segment](https://webassembly.github.io/spec/core/syntax/modules.html#syntax-elem)
        /// into the specified table.
        TableInit[(index::ElemIdx, TableIdx)] = "table.init",

        /// The
        /// [**data.drop**](https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions)
        /// instruction serves as a hint that the given
        /// [data segment](https://webassembly.github.io/spec/core/syntax/modules.html#syntax-data)
        /// will no longer be used.
        DataDrop[(index::DataIdx)] = "data.drop",
        /// The
        /// [**elem.drop**](https://webassembly.github.io/spec/core/syntax/instructions.html#table-instructions)
        /// instruction serves as a hint that the given
        /// [element segment](https://webassembly.github.io/spec/core/syntax/modules.html#syntax-elem)
        /// will no longer be used.
        ElemDrop[(index::ElemIdx)] = "elem.drop",
    }

    /// Returns `true` if the [`Instruction`] was introduced as part of the
    /// [fixed-width SIMD proposal](https://github.com/webassembly/simd), which introduced
    /// operations on 128-bit wide vectors.
    is_from_fixed_width_simd {
        /// The
        /// [**v128.load**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        /// instruction reads an 128-bit vector from memory.
        V128Load[(MemArg)] = "v128.load",

        /// [**v128.load8x8_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        V128Load8x8S[(MemArg)] = "v128.load8x8_s",
        /// [**v128.load8x8_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        V128Load8x8U[(MemArg)] = "v128.load8x8_u",
        /// [**v128.load16x4_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        V128Load16x4S[(MemArg)] = "v128.load16x4_s",
        /// [**v128.load16x4_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        V128Load16x4U[(MemArg)] = "v128.load16x4_u",
        /// [**v128.load32x2_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        V128Load32x2S[(MemArg)] = "v128.load32x2_s",
        /// [**v128.load32x2_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        V128Load32x2U[(MemArg)] = "v128.load32x2_u",

        /// [**v128.load8_splat**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        V128Load8Splat[(MemArg)] = "v128.load8_splat",
        /// [**v128.load16_splat**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        V128Load16Splat[(MemArg)] = "v128.load16_splat",
        /// [**v128.load32_splat**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        V128Load32Splat[(MemArg)] = "v128.load32_splat",
        /// [**v128.load64_splat**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        V128Load64Splat[(MemArg)] = "v128.load64_splat",

        /// [**v128.load32_zero**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        V128Load32Zero[(MemArg)] = "v128.load32_zero",
        /// [**v128.load64_zero**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        V128Load64Zero[(MemArg)] = "v128.load64_zero",

        /// The
        /// [**v128.store**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        /// instruction stores an 128-bit vector into memory.
        V128Store[(MemArg)] = "v128.store",

        /// [**v128.load8_lane**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        V128Load8Lane[(MemArg, LaneIdx)] = "v128.load8_lane",
        /// [**v128.load16_lane**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        V128Load16Lane[(MemArg, LaneIdx)] = "v128.load16_lane",
        /// [**v128.load32_lane**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        V128Load32Lane[(MemArg, LaneIdx)] = "v128.load32_lane",
        /// [**v128.load64_lane**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        V128Load64Lane[(MemArg, LaneIdx)] = "v128.load64_lane",

        /// [**v128.store8_lane**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        V128Store8Lane[(MemArg, LaneIdx)] = "v128.store8_lane",
        /// [**v128.store16_lane**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        V128Store16Lane[(MemArg, LaneIdx)] = "v128.store16_lane",
        /// [**v128.store32_lane**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        V128Store32Lane[(MemArg, LaneIdx)] = "v128.store32_lane",
        /// [**v128.store64_lane**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        V128Store64Lane[(MemArg, LaneIdx)] = "v128.store64_lane",

        /// The
        /// [**v128.const**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        /// instruction returns a 128-bit vector constant.
        V128Const[(u128)] = "v128.const",

        /// [**i8x16.shuffle**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I8x16Shuffle[([LaneIdx; 16])] = "i8x16.shuffle",

        /// [**i8x16.extract_lane_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I8x16ExtractLaneS[(LaneIdx)] = "i8x16.extract_lane_s",
        /// [**i8x16.extract_lane_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I8x16ExtractLaneU[(LaneIdx)] = "i8x16.extract_lane_u",
        /// [**i8x16.replace_lane**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I8x16ReplaceLane[(LaneIdx)] = "i8x16.replace_lane",

        /// [**i16x8.extract_lane_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I16x8ExtractLaneS[(LaneIdx)] = "i16x8.extract_lane_s",
        /// [**i16x8.extract_lane_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I16x8ExtractLaneU[(LaneIdx)] = "i16x8.extract_lane_u",
        /// [**i16x8.replace_lane**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I16x8ReplaceLane[(LaneIdx)] = "i16x8.replace_lane",

        /// [**i32x4.extract_lane**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I32x4ExtractLane[(LaneIdx)] = "i32x4.extract_lane",
        /// [**i32x4.replace_lane**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I32x4ReplaceLane[(LaneIdx)] = "i32x4.replace_lane",

        /// [**i64x2.extract_lane**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I64x2ExtractLane[(LaneIdx)] = "i64x2.extract_lane",
        /// [**i64x2.replace_lane**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I64x2ReplaceLane[(LaneIdx)] = "i64x2.replace_lane",

        /// [**f32x4.extract_lane**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        F32x4ExtractLane[(LaneIdx)] = "f32x4.extract_lane",
        /// [**f32x4.replace_lane**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        F32x4ReplaceLane[(LaneIdx)] = "f32x4.replace_lane",

        /// [**f64x2.extract_lane**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        F64x2ExtractLane[(LaneIdx)] = "f64x2.extract_lane",
        /// [**f64x2.replace_lane**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        F64x2ReplaceLane[(LaneIdx)] = "f64x2.replace_lane",

        /// [**i8x16.swizzle**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I8x16Swizzle = "i8x16.swizzle",

        /// [**i8x16.splat**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I8x16Splat = "i8x16.splat",
        /// [**i16x8.splat**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I16x8Splat = "i16x8.splat",
        /// [**i32x4.splat**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I32x4Splat = "i32x4.splat",
        /// [**i64x4.splat**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I64x2Splat = "i64x4.splat",
        /// [**f32x4.splat**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        F32x4Splat = "f32x4.splat",
        /// [**f64x2.splat**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        F64x2Splat = "f64x2.splat",

        /// [**i8x16.eq**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I8x16Eq = "i8x16.eq",
        /// [**i8x16.ne**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I8x16Ne = "i8x16.ne",
        /// [**i8x16.lt_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I8x16LtS = "i8x16.lt_s",
        /// [**i8x16.lt_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I8x16LtU = "i8x16.lt_u",
        /// [**i8x16.gt_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I8x16GtS = "i8x16.gt_s",
        /// [**i8x16.gt_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I8x16GtU = "i8x16.gt_u",
        /// [**i8x16.le_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I8x16LeS = "i8x16.le_s",
        /// [**i8x16.le_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I8x16LeU = "i8x16.le_u",
        /// [**i8x16.ge_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I8x16GeS = "i8x16.ge_s",
        /// [**i8x16.ge_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I8x16GeU = "i8x16.ge_u",

        /// [**i16x8.eq**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I16x8Eq = "i16x8.eq",
        /// [**i16x8.ne**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I16x8Ne = "i16x8.ne",
        /// [**i16x8.lt_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I16x8LtS = "i16x8.lt_s",
        /// [**i16x8.lt_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I16x8LtU = "i16x8.lt_u",
        /// [**i16x8.gt_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I16x8GtS = "i16x8.gt_s",
        /// [**i16x8.gt_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I16x8GtU = "i16x8.gt_u",
        /// [**i16x8.le_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I16x8LeS = "i16x8.le_s",
        /// [**i16x8.le_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I16x8LeU = "i16x8.le_u",
        /// [**i16x8.ge_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I16x8GeS = "i16x8.ge_s",
        /// [**i16x8.ge_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I16x8GeU = "i16x8.ge_u",

        /// [**i32x4.eq**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I32x4Eq = "i32x4.eq",
        /// [**i32x4.ne**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I32x4Ne = "i32x4.ne",
        /// [**i32x4.lt_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I32x4LtS = "i32x4.lt_s",
        /// [**i32x4.lt_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I32x4LtU = "i32x4.lt_u",
        /// [**i32x4.gt_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I32x4GtS = "i32x4.gt_s",
        /// [**i32x4.gt_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I32x4GtU = "i32x4.gt_u",
        /// [**i32x4.le_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I32x4LeS = "i32x4.le_s",
        /// [**i32x4.le_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I32x4LeU = "i32x4.le_u",
        /// [**i32x4.ge_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I32x4GeS = "i32x4.ge_s",
        /// [**i32x4.ge_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I32x4GeU = "i32x4.ge_u",

        /// [**i64x2.eq**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I64x2Eq = "i64x2.eq",
        /// [**i64x2.ne**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I64x2Ne = "i64x2.ne",
        /// [**i64x2.lt_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I64x2LtS = "i64x2.lt_s",
        /// [**i64x2.gt_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I64x2GtS = "i64x2.gt_s",
        /// [**i64x2.le_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I64x2LeS = "i64x2.le_s",
        /// [**i64x2.ge_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I64x2GeS = "i64x2.ge_s",

        /// [**f32x4.eq**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        F32x4Eq = "f32x4.eq",
        /// [**f32x4.ne**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        F32x4Ne = "f32x4.ne",
        /// [**f32x4.lt**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        F32x4Lt = "f32x4.lt",
        /// [**f32x4.gt**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        F32x4Gt = "f32x4.gt",
        /// [**f32x4.le**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        F32x4Le = "f32x4.le",
        /// [**f32x4.ge**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        F32x4Ge = "f32x4.ge",

        /// [**f64x2.eq**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        F64x2Eq = "f64x2.eq",
        /// [**f64x2.ne**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        F64x2Ne = "f64x2.ne",
        /// [**f64x2.lt**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        F64x2Lt = "f64x2.lt",
        /// [**f64x2.gt**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        F64x2Gt = "f64x2.gt",
        /// [**f64x2.le**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        F64x2Le = "f64x2.le",
        /// [**f64x2.ge**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        F64x2Ge = "f64x2.ge",

        /// [**v128.not**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        V128Not = "v128.not",
        /// [**v128.and**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        V128And = "v128.and",
        /// [**v128.andnot**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        V128AndNot = "v128.andnot",
        /// [**v128.or**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        V128Or = "v128.or",
        /// [**v128.xor**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        V128Xor = "v128.xor",
        /// [**v128.bitselect**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        V128Bitselect = "v128.bitselect",
        /// [**v128.any_true**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        V128AnyTrue = "v128.any_true",

        /// [**i8x16.abs**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I8x16Abs = "i8x16.abs",
        /// [**i8x16.neg**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I8x16Neg = "i8x16.neg",
        /// [**i8x16.popcnt**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I8x16Popcnt = "i8x16.popcnt",
        /// [**i8x16.all_true**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I8x16AllTrue = "i8x16.all_true",
        /// [**i8x16.bitmask**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I8x16Bitmask = "i8x16.bitmask",
        /// [**i8x16.narrow_i16x8_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I8x16NarrowI16x8S = "i8x16.narrow_i16x8_s",
        /// [**i8x16.narrow_i16x8_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I8x16NarrowI16x8U = "i8x16.narrow_i16x8_u",
        /// [**i8x16.shl**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I8x16Shl = "i8x16.shl",
        /// [**i8x16.shr_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I8x16ShrS = "i8x16.shr_s",
        /// [**i8x16.shr_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I8x16ShrU = "i8x16.shr_u",
        /// [**i8x16.add**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I8x16Add = "i8x16.add",
        /// [**i8x16.add_sat_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I8x16AddSatS = "i8x16.add_sat_s",
        /// [**i8x16.add_sat_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I8x16AddSatU = "i8x16.add_sat_u",
        /// [**i8x16.sub**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I8x16Sub = "i8x16.sub",
        /// [**i8x16.sub_sat_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I8x16SubSatS = "i8x16.sub_sat_s",
        /// [**i8x16.sub_sat_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I8x16SubSatU = "i8x16.sub_sat_u",
        /// [**i8x16.min_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I8x16MinS = "i8x16.min_s",
        /// [**i8x16.min_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I8x16MinU = "i8x16.min_u",
        /// [**i8x16.max_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I8x16MaxS = "i8x16.max_s",
        /// [**i8x16.max_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I8x16MaxU = "i8x16.max_u",
        /// [**i8x16.avgr_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I8x16AvgrU = "i8x16.avgr_u",

        /// [**i16x8.extadd_pairwise_i8x16_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I16x8ExtaddPairwiseI8x16S = "i16x8.extadd_pairwise_i8x16_s",
        /// [**i16x8.extadd_pairwise_i8x16_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I16x8ExtaddPairwiseI8x16U = "i16x8.extadd_pairwise_i8x16_u",
        /// [**i16x8.abs**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I16x8Abs = "i16x8.abs",
        /// [**i16x8.neg**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I16x8Neg = "i16x8.neg",
        /// [**i16x8.q15mulr_sat_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I16x8Q15MulrSatS = "i16x8.q15mulr_sat_s",
        /// [**i16x8.all_true**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I16x8AllTrue = "i16x8.all_true",
        /// [**i16x8.bitmask**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I16x8Bitmask = "i16x8.bitmask",
        /// [**i16x8.narrow_i32x4_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I16x8NarrowI32x4S = "i16x8.narrow_i32x4_s",
        /// [**i16x8.narrow_i32x4_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I16x8NarrowI32x4U = "i16x8.narrow_i32x4_u",
        /// [**i16x8.extend_low_i8x16_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I16x8ExtendLowI8x16S = "i16x8.extend_low_i8x16_s",
        /// [**i16x8.extend_high_i8x16_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I16x8ExtendHighI8x16S = "i16x8.extend_high_i8x16_s",
        /// [**i16x8.extend_low_i8x16_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I16x8ExtendLowI8x16U = "i16x8.extend_low_i8x16_u",
        /// [**i16x8.extend_high_i8x16_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I16x8ExtendHighI8x16U = "i16x8.extend_high_i8x16_u",
        /// [**i16x8.shl**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I16x8Shl = "i16x8.shl",
        /// [**i16x8.shr_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I16x8ShrS = "i16x8.shr_s",
        /// [**i16x8.shr_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I16x8ShrU = "i16x8.shr_u",
        /// [**i16x8.add**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I16x8Add = "i16x8.add",
        /// [**i16x8.add_sat_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I16x8AddSatS = "i16x8.add_sat_s",
        /// [**i16x8.add_sat_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I16x8AddSatU = "i16x8.add_sat_u",
        /// [**i16x8.sub**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I16x8Sub = "i16x8.sub",
        /// [**i16x8.sub_sat_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I16x8SubSatS = "i16x8.sub_sat_s",
        /// [**i16x8.sub_sat_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I16x8SubSatU = "i16x8.sub_sat_u",
        /// [**i16x8.mul**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I16x8Mul = "i16x8.mul",
        /// [**i16x8.min_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I16x8MinS = "i16x8.min_s",
        /// [**i16x8.min_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I16x8MinU = "i16x8.min_u",
        /// [**i16x8.max_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I16x8MaxS = "i16x8.max_s",
        /// [**i16x8.max_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I16x8MaxU = "i16x8.max_u",
        /// [**i16x8.avgr_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I16x8AvgrU = "i16x8.avgr_u",
        /// [**i16x8.extmul_low_i8x16_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I16x8ExtmulLowI8x16S = "i16x8.extmul_low_i8x16_s",
        /// [**i16x8.extmul_high_i8x16_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I16x8ExtmulHighI8x16S = "i16x8.extmul_high_i8x16_s",
        /// [**i16x8.extmul_low_i8x16_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I16x8ExtmulLowI8x16U = "i16x8.extmul_low_i8x16_u",
        /// [**i16x8.extmul_high_i8x16_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I16x8ExtmulHighI8x16U = "i16x8.extmul_high_i8x16_u",

        /// [**i32x4.extadd_pairwise_i16x8s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I32x4ExtaddPairwiseI16x8S = "i32x4.extadd_pairwise_i16x8s",
        /// [**i32x4.extadd_pairwise_i16x8u**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I32x4ExtaddPairwiseI16x8U = "i32x4.extadd_pairwise_i16x8u",
        /// [**i32x4.abs**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I32x4Abs = "i32x4.abs",
        /// [**i32x4.neg**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I32x4Neg = "i32x4.neg",
        /// [**i32x4.all_true**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I32x4AllTrue = "i32x4.all_true",
        /// [**i32x4.bitmask**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I32x4Bitmask = "i32x4.bitmask",
        /// [**i32x4.extend_low_i16x8_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I32x4ExtendLowI16x8S = "i32x4.extend_low_i16x8_s",
        /// [**i32x4.extend_high_i16x8_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I32x4ExtendHighI16x8S = "i32x4.extend_high_i16x8_s",
        /// [**i32x4.extend_low_i16x8_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I32x4ExtendLowI16x8U = "i32x4.extend_low_i16x8_u",
        /// [**i32x4.extend_high_i16x8_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I32x4ExtendHighI16x8U = "i32x4.extend_high_i16x8_u",
        /// [**i32x4.shl**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I32x4Shl = "i32x4.shl",
        /// [**i32x4.shr_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I32x4ShrS = "i32x4.shr_s",
        /// [**i32x4.shr_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I32x4ShrU = "i32x4.shr_u",
        /// [**i32x4.add**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I32x4Add = "i32x4.add",
        /// [**i32x4.sub**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I32x4Sub = "i32x4.sub",
        /// [**i32x4.mul**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I32x4Mul = "i32x4.mul",
        /// [**i32x4.min_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I32x4MinS = "i32x4.min_s",
        /// [**i32x4.min_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I32x4MinU = "i32x4.min_u",
        /// [**i32x4.max_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I32x4MaxS = "i32x4.max_s",
        /// [**i32x4.max_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I32x4MaxU = "i32x4.max_u",
        /// [**i32x4.dot_i16x8_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I32x4DotI16x8S = "i32x4.dot_i16x8_s",
        /// [**i32x4.extmul_low_i16x8_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I32x4ExtmulLowI16x8S = "i32x4.extmul_low_i16x8_s",
        /// [**i32x4.extmul_high_i16x8_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I32x4ExtmulHighI16x8S = "i32x4.extmul_high_i16x8_s",
        /// [**i32x4.extmul_low_i16x8_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I32x4ExtmulLowI16x8U = "i32x4.extmul_low_i16x8_u",
        /// [**i32x4.extmul_high_i16x8_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I32x4ExtmulHighI16x8U = "i32x4.extmul_high_i16x8_u",

        /// [**i64x2.abs**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I64x2Abs = "i64x2.abs",
        /// [**i64x2.neg**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I64x2Neg = "i64x2.neg",
        /// [**i64x2.all_true**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I64x2AllTrue = "i64x2.all_true",
        /// [**i64x2.bitmask**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I64x2Bitmask = "i64x2.bitmask",
        /// [**i64x2.extend_low_i32x4_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I64x2ExtendLowI32x4S = "i64x2.extend_low_i32x4_s",
        /// [**i64x2.extend_high_i32x4_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I64x2ExtendHighI32x4S = "i64x2.extend_high_i32x4_s",
        /// [**i64x2.extend_low_i32x4_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I64x2ExtendLowI32x4U = "i64x2.extend_low_i32x4_u",
        /// [**i64x2.extend_high_i32x4_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I64x2ExtendHighI32x4U = "i64x2.extend_high_i32x4_u",
        /// [**i64x2.shl**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I64x2Shl = "i64x2.shl",
        /// [**i64x2.shr_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I64x2ShrS = "i64x2.shr_s",
        /// [**i64x2.shr_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I64x2ShrU = "i64x2.shr_u",
        /// [**i64x2.add**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I64x2Add = "i64x2.add",
        /// [**i64x2.sub**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I64x2Sub = "i64x2.sub",
        /// [**i64x2.mul**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I64x2Mul = "i64x2.mul",
        /// [**i64x2.extmul_low_i32x4_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I64x2ExtmulLowI32x4S = "i64x2.extmul_low_i32x4_s",
        /// [**i64x2.extmul_high_i32x4_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I64x2ExtmulHighI32x4S = "i64x2.extmul_high_i32x4_s",
        /// [**i64x2.extmul_low_i32x4_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I64x2ExtmulLowI32x4U = "i64x2.extmul_low_i32x4_u",
        /// [**i64x2.extmul_high_i32x4_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I64x2ExtmulHighI32x4U = "i64x2.extmul_high_i32x4_u",

        /// [**f32x4.ceil**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        F32x4Ceil = "f32x4.ceil",
        /// [**f32x4.floor**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        F32x4Floor = "f32x4.floor",
        /// [**f32x4.trunc**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        F32x4Trunc = "f32x4.trunc",
        /// [**f32x4.nearest**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        F32x4Nearest = "f32x4.nearest",
        /// [**f32x4.abs**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        F32x4Abs = "f32x4.abs",
        /// [**f32x4.neg**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        F32x4Neg = "f32x4.neg",
        /// [**f32x4.sqrt**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        F32x4Sqrt = "f32x4.sqrt",
        /// [**f32x4.add**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        F32x4Add = "f32x4.add",
        /// [**f32x4.sub**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        F32x4Sub = "f32x4.sub",
        /// [**f32x4.mul**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        F32x4Mul = "f32x4.mul",
        /// [**f32x4.div**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        F32x4Div = "f32x4.div",
        /// [**f32x4.min**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        F32x4Min = "f32x4.min",
        /// [**f32x4.max**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        F32x4Max = "f32x4.max",
        /// [**f32x4.pmin**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        F32x4Pmin = "f32x4.pmin",
        /// [**f32x4.pmax**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        F32x4Pmax = "f32x4.pmax",

        /// [**f64x2.ceil**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        F64x2Ceil = "f64x2.ceil",
        /// [**f64x2.floor**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        F64x2Floor = "f64x2.floor",
        /// [**f64x2.trunc**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        F64x2Trunc = "f64x2.trunc",
        /// [**f64x2.nearest**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        F64x2Nearest = "f64x2.nearest",
        /// [**f64x2.abs**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        F64x2Abs = "f64x2.abs",
        /// [**f64x2.neg**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        F64x2Neg = "f64x2.neg",
        /// [**f64x2.sqrt**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        F64x2Sqrt = "f64x2.sqrt",
        /// [**f64x2.add**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        F64x2Add = "f64x2.add",
        /// [**f64x2.sub**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        F64x2Sub = "f64x2.sub",
        /// [**f64x2.mul**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        F64x2Mul = "f64x2.mul",
        /// [**f64x2.div**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        F64x2Div = "f64x2.div",
        /// [**f64x2.min**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        F64x2Min = "f64x2.min",
        /// [**f64x2.max**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        F64x2Max = "f64x2.max",
        /// [**f64x2.pmin**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        F64x2Pmin = "f64x2.pmin",
        /// [**f64x2.pmax**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        F64x2Pmax = "f64x2.pmax",

        /// [**i32x4.trunc_sat_f32x4_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I32x4TruncSatF32x4S = "i32x4.trunc_sat_f32x4_s",
        /// [**i32x4.trunc_sat_f32x4_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I32x4TruncSatF32x4U = "i32x4.trunc_sat_f32x4_u",
        /// [**f32x4.convert_i32x4_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        F32x4ConvertI32x4S = "f32x4.convert_i32x4_s",
        /// [**f32x4.convert_i32x4_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        F32x4ConvertI32x4U = "f32x4.convert_i32x4_u",
        /// [**i32x4.trunc_sat_f64x2_s_zero**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I32x4TruncSatF64x2SZero = "i32x4.trunc_sat_f64x2_s_zero",
        /// [**i32x4.trunc_sat_f64x2_u_zero**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        I32x4TruncSatF64x2UZero = "i32x4.trunc_sat_f64x2_u_zero",
        /// [**f64x2.convert_low_i32x4_s**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        F64x2ConvertLowI32x4S = "f64x2.convert_low_i32x4_s",
        /// [**f64x2.convert_low_i32x4_u**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        F64x2ConvertLowI32x4U = "f64x2.convert_low_i32x4_u",
        /// [**f32x4.demote_f64x2_zero**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        F32x4DemoteF64x2Zero = "f32x4.demote_f64x2_zero",
        /// [**f64x2.promote_low_f32x4**](https://webassembly.github.io/spec/core/syntax/instructions.html#vector-instructions)
        F64x2PromoteLowF32x4 = "f64x2.promote_low_f32x4",
    }

    /// Returns `true` if the [`Instruction`] was introduced as part of the
    /// [tail call proposal](https://github.com/WebAssembly/tail-call).
    is_from_tail_call {
        /// [**return_call**](https://webassembly.github.io/tail-call/core/syntax/instructions.html#control-instructions)
        ReturnCall[(FuncIdx)] = "return_call",
        /// [**return_call_indirect**](https://webassembly.github.io/tail-call/core/syntax/instructions.html#control-instructions)
        ReturnCallIndirect[(index::TypeIdx, TableIdx)] = "return_call_indirect",
    }

    /// Returns `true` if the [`Instruction`] is an atomic memory instruction, introduced as part
    /// of the [threads proposal](https://github.com/webassembly/threads).
    is_from_threads {
        /// [**memory.atomic.notify**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        MemoryAtomicNotify[(MemArg)] = "memory.atomic.notify",
        /// [**memory.atomic.wait32**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        MemoryAtomicWait32[(MemArg)] = "memory.atomic.wait32",
        /// [**memory.atomic.wait64**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        MemoryAtomicWait64[(MemArg)] = "memory.atomic.wait64",

        /// [**i32.atomic.load**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I32AtomicLoad[(MemArg)] = "i32.atomic.load",
        /// [**i64.atomic.load**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I64AtomicLoad[(MemArg)] = "i64.atomic.load",
        /// [**i32.atomic.load8_u**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I32AtomicLoad8U[(MemArg)] = "i32.atomic.load8_u",
        /// [**i32.atomic.load16_u**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I32AtomicLoad16U[(MemArg)] = "i32.atomic.load16_u",
        /// [**i64.atomic.load8_u**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I64AtomicLoad8U[(MemArg)] = "i64.atomic.load8_u",
        /// [**i64.atomic.load16_u**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I64AtomicLoad16U[(MemArg)] = "i64.atomic.load16_u",
        /// [**i64.atomic.load32_u**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I64AtomicLoad32U[(MemArg)] = "i64.atomic.load32_u",

        /// [**i32.atomic.store**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I32AtomicStore[(MemArg)] = "i32.atomic.store",
        /// [**i64.atomic.store**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I64AtomicStore[(MemArg)] = "i64.atomic.store",
        /// [**i32.atomic.store8_u**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I32AtomicStore8U[(MemArg)] = "i32.atomic.store8_u",
        /// [**i32.atomic.store16_u**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I32AtomicStore16U[(MemArg)] = "i32.atomic.store16_u",
        /// [**i64.atomic.store8_u**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I64AtomicStore8U[(MemArg)] = "i64.atomic.store8_u",
        /// [**i64.atomic.store16_u**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I64AtomicStore16U[(MemArg)] = "i64.atomic.store16_u",
        /// [**i64.atomic.store32_u**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I64AtomicStore32U[(MemArg)] = "i64.atomic.store32_u",

        /// [**i32.atomic.rmw.add**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I32AtomicRmwAdd[(MemArg)] = "i32.atomic.rmw.add",
        /// [**i64.atomic.rmw.add**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I64AtomicRmwAdd[(MemArg)] = "i64.atomic.rmw.add",
        /// [**i32.atomic.rmw8.add_u**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I32AtomicRmw8AddU[(MemArg)] = "i32.atomic.rmw8.add_u",
        /// [**i32.atomic.rmw16.add_u**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I32AtomicRmw16AddU[(MemArg)] = "i32.atomic.rmw16.add_u",
        /// [**i64.atomic.rmw8.add_u**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I64AtomicRmw8AddU[(MemArg)] = "i64.atomic.rmw8.add_u",
        /// [**i64.atomic.rmw16.add_u**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I64AtomicRmw16AddU[(MemArg)] = "i64.atomic.rmw16.add_u",
        /// [**i64.atomic.rmw32.add_u**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I64AtomicRmw32AddU[(MemArg)] = "i64.atomic.rmw32.add_u",

        /// [**i32.atomic.rmw.sub**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I32AtomicRmwSub[(MemArg)] = "i32.atomic.rmw.sub",
        /// [**i64.atomic.rmw.sub**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I64AtomicRmwSub[(MemArg)] = "i64.atomic.rmw.sub",
        /// [**i32.atomic.rmw8.sub_u**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I32AtomicRmw8SubU[(MemArg)] = "i32.atomic.rmw8.sub_u",
        /// [**i32.atomic.rmw16.sub_u**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I32AtomicRmw16SubU[(MemArg)] = "i32.atomic.rmw16.sub_u",
        /// [**i64.atomic.rmw8.sub_u**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I64AtomicRmw8SubU[(MemArg)] = "i64.atomic.rmw8.sub_u",
        /// [**i64.atomic.rmw16.sub_u**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I64AtomicRmw16SubU[(MemArg)] = "i64.atomic.rmw16.sub_u",
        /// [**i64.atomic.rmw32.sub_u**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I64AtomicRmw32SubU[(MemArg)] = "i64.atomic.rmw32.sub_u",

        /// [**i32.atomic.rmw.and**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I32AtomicRmwAnd[(MemArg)] = "i32.atomic.rmw.and",
        /// [**i64.atomic.rmw.and**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I64AtomicRmwAnd[(MemArg)] = "i64.atomic.rmw.and",
        /// [**i32.atomic.rmw8.and_u**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I32AtomicRmw8AndU[(MemArg)] = "i32.atomic.rmw8.and_u",
        /// [**i32.atomic.rmw16.and_u**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I32AtomicRmw16AndU[(MemArg)] = "i32.atomic.rmw16.and_u",
        /// [**i64.atomic.rmw8.and_u**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I64AtomicRmw8AndU[(MemArg)] = "i64.atomic.rmw8.and_u",
        /// [**i64.atomic.rmw16.and_u**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I64AtomicRmw16AndU[(MemArg)] = "i64.atomic.rmw16.and_u",
        /// [**i64.atomic.rmw32.and_u**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I64AtomicRmw32AndU[(MemArg)] = "i64.atomic.rmw32.and_u",

        /// [**i32.atomic.rmw.or**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I32AtomicRmwOr[(MemArg)] = "i32.atomic.rmw.or",
        /// [**i64.atomic.rmw.or**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I64AtomicRmwOr[(MemArg)] = "i64.atomic.rmw.or",
        /// [**i32.atomic.rmw8.or_u**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I32AtomicRmw8OrU[(MemArg)] = "i32.atomic.rmw8.or_u",
        /// [**i32.atomic.rmw16.or_u**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I32AtomicRmw16OrU[(MemArg)] = "i32.atomic.rmw16.or_u",
        /// [**i64.atomic.rmw8.or_u**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I64AtomicRmw8OrU[(MemArg)] = "i64.atomic.rmw8.or_u",
        /// [**i64.atomic.rmw16.or_u**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I64AtomicRmw16OrU[(MemArg)] = "i64.atomic.rmw16.or_u",
        /// [**i64.atomic.rmw32.or_u**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I64AtomicRmw32OrU[(MemArg)] = "i64.atomic.rmw32.or_u",

        /// [**i32.atomic.rmw.xor**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I32AtomicRmwXor[(MemArg)] = "i32.atomic.rmw.xor",
        /// [**i64.atomic.rmw.xor**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I64AtomicRmwXor[(MemArg)] = "i64.atomic.rmw.xor",
        /// [**i32.atomic.rmw8.xor_u**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I32AtomicRmw8XorU[(MemArg)] = "i32.atomic.rmw8.xor_u",
        /// [**i32.atomic.rmw16.xor_u**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I32AtomicRmw16XorU[(MemArg)] = "i32.atomic.rmw16.xor_u",
        /// [**i64.atomic.rmw8.xor_u**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I64AtomicRmw8XorU[(MemArg)] = "i64.atomic.rmw8.xor_u",
        /// [**i64.atomic.rmw16.xor_u**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I64AtomicRmw16XorU[(MemArg)] = "i64.atomic.rmw16.xor_u",
        /// [**i64.atomic.rmw32.xor_u**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I64AtomicRmw32XorU[(MemArg)] = "i64.atomic.rmw32.xor_u",

        /// [**i32.atomic.rmw.xchg**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I32AtomicRmwXchg[(MemArg)] = "i32.atomic.rmw.xchg",
        /// [**i64.atomic.rmw.xchg**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I64AtomicRmwXchg[(MemArg)] = "i64.atomic.rmw.xchg",
        /// [**i32.atomic.rmw8.xchg_u**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I32AtomicRmw8XchgU[(MemArg)] = "i32.atomic.rmw8.xchg_u",
        /// [**i32.atomic.rmw16.xchg_u**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I32AtomicRmw16XchgU[(MemArg)] = "i32.atomic.rmw16.xchg_u",
        /// [**i64.atomic.rmw8.xchg_u**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I64AtomicRmw8XchgU[(MemArg)] = "i64.atomic.rmw8.xchg_u",
        /// [**i64.atomic.rmw16.xchg_u**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I64AtomicRmw16XchgU[(MemArg)] = "i64.atomic.rmw16.xchg_u",
        /// [**i64.atomic.rmw32.xchg_u**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I64AtomicRmw32XchgU[(MemArg)] = "i64.atomic.rmw32.xchg_u",

        /// [**i32.atomic.rmw.cmpxchg**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I32AtomicRmwCmpxchg[(MemArg)] = "i32.atomic.rmw.cmpxchg",
        /// [**i64.atomic.rmw.cmpxchg**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I64AtomicRmwCmpxchg[(MemArg)] = "i64.atomic.rmw.cmpxchg",
        /// [**i32.atomic.rmw8.cmpxchg_u**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I32AtomicRmw8CmpxchgU[(MemArg)] = "i32.atomic.rmw8.cmpxchg_u",
        /// [**i32.atomic.rmw16.cmpxchg_u**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I32AtomicRmw16CmpxchgU[(MemArg)] = "i32.atomic.rmw16.cmpxchg_u",
        /// [**i64.atomic.rmw8.cmpxchg_u**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I64AtomicRmw8CmpxchgU[(MemArg)] = "i64.atomic.rmw8.cmpxchg_u",
        /// [**i64.atomic.rmw16.cmpxchg_u**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I64AtomicRmw16CmpxchgU[(MemArg)] = "i64.atomic.rmw16.cmpxchg_u",
        /// [**i64.atomic.rmw32.cmpxchg_u**](https://webassembly.github.io/threads/core/binary/instructions.html#atomic-memory-instructions)
        I64AtomicRmw32CmpxchgU[(MemArg)] = "i64.atomic.rmw32.cmpxchg_u",
    }

    /// Returns `true` if the [`Instruction`] was introduced as part of the
    /// [exception handling proposal](https://github.com/WebAssembly/exception-handling).
    is_exception_handling {
        /// The [**try**](https://webassembly.github.io/exception-handling/core/syntax/instructions.html#control-instructions)
        /// instruction marks the start of a block that can catch exceptions.
        Try[(BlockType)] = "try",
        /// The
        /// [**catch**](https://webassembly.github.io/exception-handling/core/syntax/instructions.html#control-instructions)
        /// instruction marks the start of an exception handler for the given
        /// [`Tag`](crate::component::Tag) for a corresponding
        /// [**try**](https://webassembly.github.io/exception-handling/core/syntax/instructions.html#control-instructions)
        /// instruction.
        Catch[(index::TagIdx)] = "catch",
        /// The
        /// [**throw**](https://webassembly.github.io/exception-handling/core/syntax/instructions.html#control-instructions)
        /// instruction create an exception with the given [`Tag`](crate::component::Tag), then
        /// [throws it](https://webassembly.github.io/exception-handling/core/exec/instructions.html#exception-handling).
        Throw[(index::TagIdx)] = "throw",
        /// The
        /// [**rethrow**](https://webassembly.github.io/exception-handling/core/syntax/instructions.html#control-instructions)
        /// throws a caught exception so that it can be handled by a different enclosing block
        /// referred to by the given label.
        Rethrow[(LabelIdx)] = "rethrow",
        /// The
        /// [**catch_all**](https://webassembly.github.io/exception-handling/core/syntax/instructions.html#control-instructions)
        /// instruction marks the start of the handler for uncaught exceptions in the block of a corresponding
        /// [**try**](https://webassembly.github.io/exception-handling/core/syntax/instructions.html#control-instructions)
        /// instruction.
        CatchAll = "catch_all",
        /// The
        /// [**delegate**](https://webassembly.github.io/exception-handling/core/syntax/instructions.html#control-instructions)
        /// instruction delegates exception handling within the block of an associated
        /// [**try**](https://webassembly.github.io/exception-handling/core/syntax/instructions.html#control-instructions)
        /// instruction to the [**catch**]/[**catch_all**] handler associated with the given label.
        ///
        /// [**catch**]: https://webassembly.github.io/exception-handling/core/syntax/instructions.html#control-instructions
        /// [**catch_all**]: https://webassembly.github.io/exception-handling/core/syntax/instructions.html#control-instructions
        Delegate[(LabelIdx)] = "delegate",
    }
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
