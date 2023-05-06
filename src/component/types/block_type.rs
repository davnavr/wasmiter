use crate::component::{TypeIdx, ValType};
use crate::parser::input::Input;
use crate::parser::{Parser, Result, ResultExt};

/// Represents a
/// [`blocktype`](https://webassembly.github.io/spec/core/binary/instructions.html#binary-blocktype),
/// which describes the types of the inputs and results of a
/// [block](https://webassembly.github.io/spec/core/binary/instructions.html#control-instructions).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum BlockType {
    /// Indicates a block has no outputs.
    None,
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

impl BlockType {
    pub(crate) fn parse<I: Input>(parser: &mut Parser<I>) -> Result<Self> {
        let value = parser.leb128_s64().context("block type tag or index")?;
        Ok(match value {
            -64 => Self::None,
            -1 => Self::from(ValType::I32),
            -2 => Self::from(ValType::I64),
            -3 => Self::from(ValType::F32),
            -4 => Self::from(ValType::F64),
            -5 => Self::from(ValType::V128),
            -16 => Self::from(ValType::FuncRef),
            -17 => Self::from(ValType::ExternRef),
            _ if value < 0 => {
                return Err(crate::parser_bad_format!(
                    "{value} is not a valid value type or block type"
                ))
            }
            _ => Self::from(
                u32::try_from(value)
                    .ok()
                    .and_then(|index| TypeIdx::from_u32(index))
                    .ok_or_else(|| {
                        crate::parser_bad_format!("{value} is too large to be a valid type index")
                    })?,
            ),
        })
    }
}
