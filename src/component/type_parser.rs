use crate::{
    component,
    input::Input,
    parser::{self, leb128, Context, Error, ErrorKind, MixedResult, Parsed, ResultExt},
    types::{self, BlockType, GlobalMutability, IdxType, Limits, TableType, ValType},
};

/// Parses a [`BlockType`].
pub fn block_type<I: Input>(offset: &mut u64, input: I) -> Parsed<BlockType> {
    #[cold]
    #[inline(never)]
    fn not_a_valid_type(value: i64) -> Error {
        Error::new(ErrorKind::InvalidFormat).with_context(Context::from_closure(move |f| {
            write!(f, "{value} is not a valid value type or block type")
        }))
    }

    let value = leb128::s64(offset, input).context("block type tag or index")?;
    Ok(match value {
        -64 => BlockType::Empty,
        -1 => BlockType::from(ValType::I32),
        -2 => BlockType::from(ValType::I64),
        -3 => BlockType::from(ValType::F32),
        -4 => BlockType::from(ValType::F64),
        -5 => BlockType::from(ValType::V128),
        -16 => BlockType::from(ValType::FuncRef),
        -17 => BlockType::from(ValType::ExternRef),
        _ if value < 0 => return Err(not_a_valid_type(value)),
        _ => BlockType::from(crate::index::TypeIdx::try_from(value as u64)?),
    })
}

/// Parses a [`ValType`].
///
/// Returns an error if some other [`BlockType`] is parsed instead.
pub fn val_type<I: Input>(offset: &mut u64, input: I) -> Parsed<ValType> {
    #[inline(never)]
    #[cold]
    fn empty_block_type() -> Error {
        Error::new(ErrorKind::EmptyBlockTypeInValType)
    }

    #[inline(never)]
    #[cold]
    fn unexpected_type_index(index: crate::index::TypeIdx) -> Error {
        Error::new(ErrorKind::TypeIndexInValType(index))
    }

    match block_type(offset, input)? {
        BlockType::Empty => Err(empty_block_type()),
        BlockType::Index(index) => Err(unexpected_type_index(index)),
        BlockType::Inline(value_type) => Ok(value_type),
    }
}

/// Parses a [`RefType`](types::RefType).
///
/// Returns an error if some other [`ValType`] is parsed instead.
pub fn ref_type<I: Input>(offset: &mut u64, input: I) -> Parsed<types::RefType> {
    #[inline(never)]
    #[cold]
    fn not_a_ref_type(actual: ValType) -> Error {
        Error::new(ErrorKind::ExpectedRefType(actual))
    }

    let value_type = val_type(offset, input)?;
    if let Some(ref_type) = value_type.try_to_ref_type() {
        Ok(ref_type)
    } else {
        Err(not_a_ref_type(value_type))
    }
}

/// Parses a [`TableType`].
pub fn table_type<I: Input>(offset: &mut u64, input: &I) -> Parsed<TableType> {
    Ok(TableType::new(
        ref_type(offset, input).context("table element type")?,
        limits(offset, input).context("table limits")?,
    ))
}

/// Parses a global [`mut`](https://webassembly.github.io/spec/core/binary/types.html#binary-mut) value.
pub fn global_mutability<I: Input>(offset: &mut u64, input: I) -> Parsed<GlobalMutability> {
    #[inline(never)]
    #[cold]
    fn bad_mutability_flag(flag: u8) -> Error {
        Error::new(ErrorKind::BadGlobalMutability(flag))
    }

    match parser::one_byte_exact(offset, input).context("global mutability flag")? {
        0 => Ok(GlobalMutability::Constant),
        1 => Ok(GlobalMutability::Variable),
        bad => Err(bad_mutability_flag(bad)),
    }
}

/// Parses a [`GlobalType`](types::GlobalType)
pub fn global_type<I: Input>(offset: &mut u64, input: &I) -> Parsed<types::GlobalType> {
    let value_type = val_type(offset, input).context("global type")?;
    let mutability = global_mutability(offset, input)?;
    Ok(types::GlobalType::new(mutability, value_type))
}

/// Parses a [`MemType`](types::MemType).
#[inline]
pub fn mem_type<I: Input>(offset: &mut u64, input: &I) -> Parsed<types::MemType> {
    limits(offset, input)
}

/// Parses [`Limits`].
pub fn limits<I: Input>(offset: &mut u64, input: &I) -> Parsed<Limits> {
    let flag = parser::one_byte_exact(offset, input).context("parsing limit flag")?;

    if flag & (!0b111) != 0 {
        #[inline(never)]
        #[cold]
        fn bad_limit_flags(flags: u8) -> Error {
            Error::new(ErrorKind::BadLimitFlags(flags))
        }

        return Err(bad_limit_flags(flag));
    }

    const USE_MEMORY_64: u8 = 0b100;

    let index_type = if flag & USE_MEMORY_64 == 0 {
        IdxType::I32
    } else {
        IdxType::I64
    };

    // 64-bit shared memory should use 64-bit limits, see github.com/WebAssembly/memory64/issues/21
    let minimum = match index_type {
        IdxType::I32 => {
            u64::from(leb128::u32(offset, input).context("parsing 32-bit limit minimum")?)
        }
        IdxType::I64 => leb128::u64(offset, input).context("parsing 64-bit limit minimum")?,
    };

    const HAS_MAXIMUM: u8 = 1;

    let maximum = if flag & HAS_MAXIMUM == 0 {
        None
    } else {
        Some(match index_type {
            IdxType::I32 => u64::from(leb128::u32(offset, input).context("32-bit limit maximum")?),
            IdxType::I64 => leb128::u64(offset, input).context("64-bit limit maximum")?,
        })
    };

    const IS_SHARED: u8 = 0b10;

    let share = if flag & IS_SHARED == 0 {
        types::Sharing::Unshared
    } else {
        types::Sharing::Shared
    };

    #[inline(never)]
    #[cold]
    fn limit_maximum_greater_than_minimum(minimum: u64, maximum: Option<u64>) -> Error {
        Error::new(ErrorKind::InvalidFormat).with_context(Context::from_closure(move |f| {
            write!(
                f,
                "the limit maximum {} cannot be less than the minimum {minimum}",
                maximum.unwrap()
            )
        }))
    }

    if let Some(limits) = Limits::new(minimum, maximum, share, index_type) {
        Ok(limits)
    } else {
        Err(limit_maximum_greater_than_minimum(minimum, maximum))
    }
}

/// Parses a WebAssembly [function type], which specifies the parameter and result types of a
/// function.
///
/// This is used, for example, to parse each entry in a
/// [`TypesComponent`](component::TypesComponent).
///
/// [function type]: https://webassembly.github.io/spec/core/syntax/types.html#function-types
pub fn func_type<I, E, Y, Z, P, R>(
    offset: &mut u64,
    input: &I,
    parameter_types: P,
    result_types: R,
) -> MixedResult<Z, E>
where
    I: Input,
    P: FnOnce(&mut component::ResultType<&mut u64, &I>) -> MixedResult<Y, E>,
    R: FnOnce(Y, &mut component::ResultType<&mut u64, &I>) -> MixedResult<Z, E>,
{
    let tag = parser::one_byte_exact(offset, input).context("function type")?;
    if tag != 0x60 {
        #[inline(never)]
        #[cold]
        fn bad_tag(tag: u8) -> Error {
            Error::new(ErrorKind::BadFuncTypeTag(tag))
        }

        return Err(bad_tag(tag).into());
    }

    let mut parameters = component::ResultType::parse(offset, input)?;
    let result_types_closure_argument = parameter_types(&mut parameters)?;
    let (offset, _) = parameters.finish()?;
    let mut results = component::ResultType::parse(offset, input)?;
    let ret = result_types(result_types_closure_argument, &mut results)?;
    results.finish()?;
    Ok(ret)
}
