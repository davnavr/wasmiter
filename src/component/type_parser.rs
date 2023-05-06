use crate::component::{BlockType, TypeIdx, ValType};
use crate::parser::input::Input;
use crate::parser::{Error, Parser, Result, ResultExt};

pub(crate) fn parse_block_type<I: Input>(parser: &mut Parser<I>) -> Result<BlockType> {
    let value = parser.leb128_s64().context("block type tag or index")?;
    Ok(match value {
        -64 => BlockType::Empty,
        -1 => BlockType::from(ValType::I32),
        -2 => BlockType::from(ValType::I64),
        -3 => BlockType::from(ValType::F32),
        -4 => BlockType::from(ValType::F64),
        -5 => BlockType::from(ValType::V128),
        -16 => BlockType::from(ValType::FuncRef),
        -17 => BlockType::from(ValType::ExternRef),
        _ if value < 0 => {
            return Err(crate::parser_bad_format!(
                "{value} is not a valid value type or block type"
            ))
        }
        _ => BlockType::from(
            u32::try_from(value)
                .ok()
                .and_then(|index| TypeIdx::from_u32(index))
                .ok_or_else(|| {
                    crate::parser_bad_format!("{value} is too large to be a valid type index")
                })?,
        ),
    })
}

pub(crate) fn parse_val_type(parser: &mut Parser<impl Input>) -> Result<ValType> {
    match parse_block_type(parser)? {
        BlockType::Empty => Err(Error::bad_format().with_context("empty type is not allowed here")),
        BlockType::Index(index) => Err(crate::parser_bad_format!(
            "expected value type but got type index {index:?}"
        )),
        BlockType::Inline(value_type) => Ok(value_type),
    }
}
