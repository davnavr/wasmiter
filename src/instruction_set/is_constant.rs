use crate::instruction_set::Instruction;

/// Indicates whether an [`Instruction`] can be used in a
/// [constant expression](https://webassembly.github.io/spec/core/valid/instructions.html#valid-constant).
///
/// See the documentation for [`Instruction::valid_in_constant`] for more information.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum IsConstant {
    /// The instruction can always be used in a constant expression.
    Constant,
    /// The instruction can be used in a constant expression **only if** the value of the given
    /// global is also a constant.
    Global(crate::index::GlobalIdx),
    /// The instruction **cannot** be used in a constant expression.
    NotConstant,
}

impl<B: crate::bytes::Bytes> Instruction<'_, B> {
    /// Returns `true` if the [`Instruction`] can be used in a
    /// [constant expression](https://webassembly.github.io/spec/core/valid/instructions.html#valid-constant).
    ///
    /// To also check for basic arithmetic operations, see
    /// [`valid_in_extended_constant`](Instruction::valid_in_extended_constant).
    pub const fn valid_in_constant(&self) -> IsConstant {
        match self {
            Self::I32Const(_)
            | Self::I64Const(_)
            | Self::F32Const(_)
            | Self::F64Const(_)
            | Self::V128Const(_)
            | Self::RefNull(_)
            | Self::RefFunc(_) => IsConstant::Constant,
            Self::GlobalGet(index) => IsConstant::Global(*index),
            _ => IsConstant::NotConstant,
        }
    }

    /// Returns `true` if the [`Instruction`] can be used in a
    /// [constant expression](https://webassembly.github.io/extended-const/core/valid/instructions.html#constant-expressions),
    /// which includes instructions listed in the
    /// [extended constant expressions proposal](https://github.com/WebAssembly/extended-const).
    pub const fn valid_in_extended_constant(&self) -> IsConstant {
        match self {
            Self::I32Add
            | Self::I32Sub
            | Self::I32Mul
            | Self::I64Add
            | Self::I64Sub
            | Self::I64Mul => IsConstant::Constant,
            _ => self.valid_in_constant(),
        }
    }
}
