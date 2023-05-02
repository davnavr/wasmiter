//! Low-level types and functions for parsing.

mod error;
mod result_ext;

pub mod input;

pub use error::{Context, Error, ErrorKind};
pub use result_ext::ResultExt;

use input::Input;

/// Result type used when parsing input.
pub type Result<T> = core::result::Result<T, Error>;

macro_rules! parser_bad_format {
    ($($arg:tt)*) => {{
        let err;

        #[cfg(feature = "alloc")]
        {
            err = Error::bad_format().with_context(alloc::format!($($arg)*));
        }

        #[cfg(not(feature = "alloc"))]
        {
            err = Error::bad_format();
        }

        err
    }};
}

trait IntegerEncoding:
    From<u8> + Default + core::ops::BitOrAssign + core::ops::Shl<u8, Output = Self>
{
    /// The maximum number of bytes that a value is allowed to be encoded in.
    ///
    /// According to the
    /// [WebAssembly specification](https://webassembly.github.io/spec/core/binary/values.html#integers),
    /// this should be equal to `ceil(BITS / 7)`.
    const MAX_LENGTH: u8;

    /// The bit-width of the integer.
    const BITS: u8;

    /// A buffer to contain the bytes encoding the integer.
    ///
    /// Should have a length equal to `MAX_LENGTH`.
    type Buffer: AsRef<[u8]> + AsMut<[u8]> + Default + IntoIterator<Item = u8>;
}

impl IntegerEncoding for u32 {
    const MAX_LENGTH: u8 = 5;
    const BITS: u8 = 32;
    type Buffer = [u8; Self::MAX_LENGTH as usize];
}

impl IntegerEncoding for u64 {
    const MAX_LENGTH: u8 = 10;
    const BITS: u8 = 64;
    type Buffer = [u8; Self::MAX_LENGTH as usize];
}

/// Parses a stream of bytes.
#[derive(Debug)]
pub struct Parser<I: Input> {
    input: I,
}

impl<I: Input> Parser<I> {
    /// Creates a new parser with the specified [`Input`].
    pub fn new(input: I) -> Self {
        Self { input }
    }

    fn leb128_unsigned<N: IntegerEncoding>(&mut self) -> Result<N> {
        let mut buffer = N::Buffer::default();
        let mut value = N::default();
        self.input.peek(buffer.as_mut())?;

        let mut more = true;
        for (index, byte) in buffer.into_iter().enumerate() {
            let i = index as u8;

            let bits = byte & 0x7F;
            more = byte & 0x80 == 0x80;

            let amount;

            if i == N::MAX_LENGTH - 1 {
                amount = N::BITS - (7 * i);

                debug_assert!(amount < 8);

                let leading_zeroes = bits.leading_zeros() as u8;
                if leading_zeroes < 8 - amount {
                    // Overflow, the number of value bits will not fit in the destination
                    return Err(parser_bad_format!(
                        "encoded value requires {} bits, which cannot fit in the destination",
                        (7 * i) + (8 - leading_zeroes)
                    ));
                }
            } else {
                amount = 7u8;
            }

            value |= N::from(bits) << amount;

            if !more {
                break;
            }
        }

        if more {
            return Err(parser_bad_format!(
                "integer can only be encoded in {} bytes",
                N::MAX_LENGTH
            ));
        }

        Ok(value)
    }

    /// Attempts to a parse an unsigned 32-bit integer encoded in
    /// [*LEB128* format](https://webassembly.github.io/spec/core/binary/values.html#integers).
    pub fn leb128_u32(&mut self) -> Result<u32> {
        self.leb128_unsigned().context("could not parse u32")
    }

    /// Attempts to parse a [`u32`] in *LEB128* format, interpreting the result as a [`usize`].
    ///
    /// This method is meant to parse
    /// [vector lengths](https://webassembly.github.io/spec/core/binary/conventions.html#vectors),
    /// which the specification currently limits to a 32-bit amount.
    ///
    /// See [`Parser::leb128_u32`] for more information.
    pub fn leb128_usize(&mut self) -> Result<usize> {
        let length = self
            .leb128_unsigned::<u32>()
            .context("could not parse length")?;

        usize::try_from(length).map_err(|_| parser_bad_format!("length ({length}) is too large"))
    }
}
