use crate::parser::{input::Input, Parser, Result, ResultExt};

/// Describes the minimum and maximum number of pages in a memory or elements in a table.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Limits {
    minimum: u32,
    maximum: Option<u32>,
}

impl Limits {
    const fn new(minimum: u32, maximum: Option<u32>) -> Option<Self> {
        match maximum {
            Some(max) if max < minimum => None,
            _ => Some(Self { minimum, maximum }),
        }
    }

    /// The minimum.
    #[inline]
    pub const fn minimum(&self) -> u32 {
        self.minimum
    }

    /// The optional minimum.
    #[inline]
    pub const fn maximum(&self) -> Option<u32> {
        self.maximum
    }

    pub(in crate::component) fn parse<I: Input>(parser: &mut Parser<I>) -> Result<Self> {
        let mut flag = 0u8;
        parser
            .bytes_exact(core::slice::from_mut(&mut flag))
            .context("limit flag")?;
        let minimum = parser.leb128_u32().context("limit minimum")?;
        let maximum = match flag {
            0 => None,
            1 => Some(parser.leb128_u32().context("limit maximum")?),
            _ => {
                return Err(crate::parser_bad_format!(
                    "{flag:#02X} is not a known limit flag"
                ))
            }
        };

        Self::new(minimum, maximum).ok_or_else(|| {
            crate::parser_bad_format!(
                "the limit maximum {} cannot be less than the minimum {minimum}",
                maximum.unwrap()
            )
        })
    }
}

/// A
/// [WebAssembly memory type](https://webassembly.github.io/spec/core/binary/types.html#memory-types),
/// with a [`Limits`] value indicating the minimum and maximum number of pages.
pub type MemType = Limits;
