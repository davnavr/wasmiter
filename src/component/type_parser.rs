use crate::component::{self, BlockType, ValType};
use crate::parser::input::Input;
use crate::parser::{self, Decoder, Error, Result, ResultExt};

impl<I: Input> Decoder<I> {
    /// Parses a [`BlockType`].
    pub fn block_type(&mut self) -> Result<BlockType> {
        let value = self.leb128_s64().context("block type tag or index")?;
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
    pub fn val_type(&mut self) -> Result<ValType> {
        match self.block_type()? {
            BlockType::Empty => {
                Err(Error::bad_format()
                    .with_context("expected value type but got empty block type"))
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
    pub fn ref_type(&mut self) -> Result<component::RefType> {
        let value_type = self.val_type()?;
        value_type.try_to_ref_type().ok_or_else(|| {
            crate::parser_bad_format!("expected reference type but got {value_type}")
        })
    }

    /// Parses a [`TableType`](component::TableType).
    pub fn table_type(&mut self) -> Result<component::TableType> {
        Ok(component::TableType::new(
            self.ref_type().context("table element type")?,
            self.limits().context("table limits")?,
        ))
    }

    /// Parses a global [`mut`](https://webassembly.github.io/spec/core/binary/types.html#binary-mut) value.
    pub fn global_mutability(&mut self) -> Result<component::GlobalMutability> {
        match self.one_byte_exact().context("global mutability flag")? {
            0 => Ok(component::GlobalMutability::Constant),
            1 => Ok(component::GlobalMutability::Variable),
            bad => Err(crate::parser_bad_format!(
                "{bad:#02X} is not a valid global mutability flag"
            )),
        }
    }

    /// Parses a [`GlobalType`](component::GlobalType)
    pub fn global_type(&mut self) -> Result<component::GlobalType> {
        let value_type = self.val_type().context("global type")?;
        let mutability = self.global_mutability()?;
        Ok(component::GlobalType::new(mutability, value_type))
    }

    /// Parses a [`MemType`](component::MemType).
    #[inline]
    pub fn mem_type(&mut self) -> Result<component::MemType> {
        self.limits()
    }

    /// Parses [`Limits`](component::Limits).
    pub fn limits(&mut self) -> Result<component::Limits> {
        let flag = self.one_byte_exact().context("limit flag")?;
        let minimum = self.leb128_u32().context("limit minimum")?;
        let maximum = match flag {
            0 => None,
            1 => Some(self.leb128_u32().context("limit maximum")?),
            _ => {
                return Err(crate::parser_bad_format!(
                    "{flag:#02X} is not a known limit flag"
                ))
            }
        };

        component::Limits::new(minimum, maximum).ok_or_else(|| {
            crate::parser_bad_format!(
                "the limit maximum {} cannot be less than the minimum {minimum}",
                maximum.unwrap()
            )
        })
    }

    /// Parses a
    /// [WebAssembly function type](https://webassembly.github.io/spec/core/syntax/types.html#function-types),
    /// which specifies the parameter and result types of a function.
    pub fn func_type<P, R>(&mut self, parameter_types: P, result_types: R) -> Result<()>
    where
        P: FnOnce(&mut component::ResultType<&mut I>) -> Result<()>,
        R: FnOnce(&mut component::ResultType<&mut I>) -> Result<()>,
    {
        let tag = self.one_byte_exact().context("function type")?;
        if tag != 0x60 {
            return Err(crate::parser_bad_format!(
                "expected function type (0x60) but got {tag:#02X}"
            ));
        }

        let mut parameters = component::ResultType::new(self.by_ref(), Default::default())?;
        parameter_types(&mut parameters)?;
        parameters.finish()?;
        let mut results = component::ResultType::new(self.by_ref(), Default::default())?;
        result_types(&mut results)?;
        results.finish()
    }
}

macro_rules! type_parse_impls {
    ($($ty:ty => $name:ident,)*) => {$(
        impl parser::Parse for parser::SimpleParse<$ty> {
            type Output = $ty;

            #[inline]
            fn parse<I: Input>(&mut self, input: &mut Decoder<I>) -> Result<$ty> {
                input.$name()
            }
        }
    )*};
}

type_parse_impls! {
    ValType => val_type,
}
