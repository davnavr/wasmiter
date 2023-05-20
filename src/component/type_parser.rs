use crate::bytes::Bytes;
use crate::component::{self, BlockType, ValType};
use crate::parser::{self, leb128, Error, Result, ResultExt};

/// Parses a [`BlockType`].
pub fn block_type<B: Bytes>(offset: &mut u64, bytes: B) -> Result<BlockType> {
    let value = leb128::s64(offset, bytes).context("block type tag or index")?;
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
        _ => BlockType::from(component::TypeIdx::try_from(value as u64)?),
    })
}

/// Parses a [`ValType`].
///
/// Returns an error if some other [`BlockType`] is parsed instead.
pub fn val_type<B: Bytes>(offset: &mut u64, bytes: B) -> Result<ValType> {
    match block_type(offset, bytes)? {
        BlockType::Empty => {
            Err(Error::bad_format().with_context("expected value type but got empty block type"))
        }
        BlockType::Index(index) => Err(crate::parser_bad_format!(
            "expected value type but got type index {index:?}"
        )),
        BlockType::Inline(value_type) => Ok(value_type),
    }
}

/// Parses a [`RefType`](component::RefType).
///
/// Returns an error if some other [`ValType`] is parsed instead.
pub fn ref_type<B: Bytes>(offset: &mut u64, bytes: B) -> Result<component::RefType> {
    let value_type = val_type(offset, bytes)?;
    value_type
        .try_to_ref_type()
        .ok_or_else(|| crate::parser_bad_format!("expected reference type but got {value_type}"))
}

/// Parses a [`TableType`](component::TableType).
pub fn table_type<B: Bytes>(offset: &mut u64, bytes: &B) -> Result<component::TableType> {
    Ok(component::TableType::new(
        ref_type(offset, bytes).context("table element type")?,
        limits(offset, bytes).context("table limits")?,
    ))
}

/// Parses a global [`mut`](https://webassembly.github.io/spec/core/binary/types.html#binary-mut) value.
pub fn global_mutability<B: Bytes>(
    offset: &mut u64,
    bytes: B,
) -> Result<component::GlobalMutability> {
    match parser::one_byte_exact(offset, bytes).context("global mutability flag")? {
        0 => Ok(component::GlobalMutability::Constant),
        1 => Ok(component::GlobalMutability::Variable),
        bad => Err(crate::parser_bad_format!(
            "{bad:#02X} is not a valid global mutability flag"
        )),
    }
}

/// Parses a [`GlobalType`](component::GlobalType)
pub fn global_type<B: Bytes>(offset: &mut u64, bytes: &B) -> Result<component::GlobalType> {
    let value_type = val_type(offset, bytes).context("global type")?;
    let mutability = global_mutability(offset, bytes)?;
    Ok(component::GlobalType::new(mutability, value_type))
}

/// Parses a [`MemType`](component::MemType).
#[inline]
pub fn mem_type<B: Bytes>(offset: &mut u64, bytes: &B) -> Result<component::MemType> {
    limits(offset, bytes)
}

/// Parses [`Limits`](component::Limits).
pub fn limits<B: Bytes>(offset: &mut u64, bytes: &B) -> Result<component::Limits> {
    let flag = parser::one_byte_exact(offset, bytes).context("limit flag")?;
    let minimum = leb128::u32(offset, bytes).context("32-bit limit minimum")?;
    let maximum = match flag {
        0 | 2 => None,
        1 | 3 => Some(leb128::u32(offset, bytes).context("32-bit limit maximum")?),
        _ => {
            return Err(crate::parser_bad_format!(
                "{flag:#02X} is not a known limit flag"
            ))
        }
    };

    // Note that only 2 and 3 is introduced in the threads proposal overview
    let share = if matches!(flag, 2 | 3 | 6 | 7) {
        component::Sharing::Shared
    } else {
        component::Sharing::Unshared
    };

    component::Limits::new(minimum, maximum, share).ok_or_else(|| {
        crate::parser_bad_format!(
            "the limit maximum {} cannot be less than the minimum {minimum}",
            maximum.unwrap()
        )
    })
}

/// Parses a
/// [WebAssembly function type](https://webassembly.github.io/spec/core/syntax/types.html#function-types),
/// which specifies the parameter and result types of a function.
pub fn func_type<Y, Z, B: Bytes, P, R>(
    offset: &mut u64,
    bytes: &B,
    parameter_types: P,
    result_types: R,
) -> Result<Z>
where
    P: FnOnce(&mut component::ResultType<&mut u64, &B>) -> Result<Y>,
    R: FnOnce(Y, &mut component::ResultType<&mut u64, &B>) -> Result<Z>,
{
    let tag = parser::one_byte_exact(offset, bytes).context("function type")?;
    if tag != 0x60 {
        return Err(crate::parser_bad_format!(
            "expected function type (0x60) but got {tag:#02X}"
        ));
    }

    let offset_reborrow: &mut u64 = offset;
    let mut parameters = component::ResultType::new(offset_reborrow, bytes, Default::default())?;
    let result_types_closure_argument = parameter_types(&mut parameters)?;
    parameters.finish()?;
    let mut results = component::ResultType::new(offset, bytes, Default::default())?;
    let ret = result_types(result_types_closure_argument, &mut results)?;
    results.finish()?;
    Ok(ret)
}

macro_rules! type_parse_impls {
    ($($ty:ty => $name:ident,)*) => {$(
        impl parser::Parse for parser::SimpleParse<$ty> {
            type Output = $ty;

            #[inline]
            fn parse<B: Bytes>(&mut self, offset: &mut u64, bytes: B) -> Result<$ty> {
                $name(offset, &bytes)
            }
        }
    )*};
}

type_parse_impls! {
    ValType => val_type,
    component::TableType => table_type,
    component::Limits => limits,
}
