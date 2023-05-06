use crate::component::{self, BlockType, TypeIdx, ValType};
use crate::parser::input::Input;
use crate::parser::{Error, Parser, Result, ResultExt};

pub(crate) fn parse_block_type(parser: &mut Parser<impl Input>) -> Result<BlockType> {
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
                .and_then(TypeIdx::from_u32)
                .ok_or_else(|| {
                    crate::parser_bad_format!("{value} is too large to be a valid type index")
                })?,
        ),
    })
}

pub(crate) fn parse_val_type(parser: &mut Parser<impl Input>) -> Result<ValType> {
    match parse_block_type(parser)? {
        BlockType::Empty => {
            Err(Error::bad_format().with_context("expected value type but got empty block type"))
        }
        BlockType::Index(index) => Err(crate::parser_bad_format!(
            "expected value type but got type index {index:?}"
        )),
        BlockType::Inline(value_type) => Ok(value_type),
    }
}

fn parse_ref_type(parser: &mut Parser<impl Input>) -> Result<component::RefType> {
    let value_type = parse_val_type(parser)?;
    value_type
        .try_to_ref_type()
        .ok_or_else(|| crate::parser_bad_format!("expected reference type but got {value_type}"))
}

pub(crate) fn parse_table_type(parser: &mut Parser<impl Input>) -> Result<component::TableType> {
    Ok(component::TableType::new(
        parse_ref_type(parser).context("table element type")?,
        component::Limits::parse(parser).context("table limits")?,
    ))
}
