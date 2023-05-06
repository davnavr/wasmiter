use crate::component::{self, BlockType, TypeIdx, ValType};
use crate::parser::{self, Error, Parser, Result, ResultExt};

impl<I: parser::input::Input> Parser<I> {
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

    /// Parses a [`MemType`](component::MemType).
    #[inline]
    pub fn mem_type(&mut self) -> Result<component::MemType> {
        self.limits()
    }

    /// Parses [`Limits`](component::Limits).
    pub fn limits(&mut self) -> Result<component::Limits> {
        let mut flag = 0u8;
        self.bytes_exact(core::slice::from_mut(&mut flag))
            .context("limit flag")?;
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
}
