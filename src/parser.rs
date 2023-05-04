//! Low-level types and functions for parsing.

mod error;
mod result_ext;

pub mod input;

pub use error::{Context, Error, ErrorKind};
pub use result_ext::ResultExt;

use input::Input;

/// Result type used when parsing input.
pub type Result<T> = core::result::Result<T, Error>;

#[macro_export]
#[doc(hidden)]
macro_rules! parser_bad_format {
    ($($arg:tt)*) => {{
        #[cfg(not(feature = "alloc"))]
        let err = crate::parser::Error::bad_format();
        #[cfg(feature = "alloc")]
        let err = crate::parser::Error::bad_format().with_context(alloc::format!($($arg)*));
        err
    }};
}

macro_rules! parser_bad_input {
    ($error:expr, $($arg:tt)*) => {{
        #[cfg(not(feature = "alloc"))]
        let err;
        #[cfg(feature = "alloc")]
        let mut err;

        err = <Error as From<input::Error>>::from($error);

        #[cfg(feature = "alloc")]
        {
            err = err.with_context(alloc::format!($($arg)*));
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
    pub fn new<S: input::IntoInput<In = I>>(input: S) -> Self {
        Self {
            input: input.into_input(),
        }
    }

    pub(crate) fn fork(&self) -> Result<Parser<I::Fork>> {
        Ok(Parser::new(self.input.fork()?))
    }

    #[inline]
    pub(crate) fn position(&self) -> Result<u64> {
        self.input.position().map_err(Error::from)
    }

    fn leb128_unsigned<N: IntegerEncoding>(&mut self) -> Result<N> {
        let mut buffer = N::Buffer::default();
        let mut value = N::default();
        self.input.peek(buffer.as_mut())?;

        let mut more = true;
        let mut i = 0u8;
        for byte in buffer.into_iter() {
            let bits = byte & 0x7F;
            more = byte & 0x80 == 0x80;

            let shift = 7u8 * i;

            // Check for overflowing bits in last byte
            if i == N::MAX_LENGTH - 1 {
                let leading_zeroes = bits.leading_zeros() as u8;
                if leading_zeroes < 8 - (N::BITS - shift) {
                    // Overflow, the number of value bits will not fit in the destination
                    return Err(parser_bad_format!(
                        "encoded value requires {} bits, which cannot fit in the destination",
                        shift + (8 - leading_zeroes)
                    ));
                }
            }

            debug_assert!(shift <= N::BITS);

            value |= N::from(bits) << shift;
            i += 1;

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

        self.input.read(u64::from(i))?;
        Ok(value)
    }

    /// Attempts to a parse an unsigned 32-bit integer encoded in
    /// [*LEB128* format](https://webassembly.github.io/spec/core/binary/values.html#integers).
    pub fn leb128_u32(&mut self) -> Result<u32> {
        // TODO: less branches variant? have buffer = [0u8; 5] // 0 means end, even if peek does not fill buffer completely
        // copy-paste code that reads from buffer[0], buffer[1], etc. Only return at the very end?
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

    /// Attempts to a parse an unsigned 64-bit integer encoded in
    /// [*LEB128* format](https://webassembly.github.io/spec/core/binary/values.html#integers).
    pub fn leb128_u64(&mut self) -> Result<u64> {
        self.leb128_unsigned().context("could not parse u64")
    }

    pub(crate) fn bytes(&mut self, buffer: &mut [u8]) -> Result<usize> {
        self.input
            .take(buffer)
            .map_err(|_e| parser_bad_input!(_e, "could not read {} bytes", buffer.len()))
    }

    pub(crate) fn bytes_exact(&mut self, buffer: &mut [u8]) -> Result<()> {
        self.input
            .take_exact(buffer)
            .map_err(|_e| parser_bad_input!(_e, "expected {} bytes", buffer.len()))
    }

    pub(crate) fn skip_exact(&mut self, amount: u64) -> Result<()> {
        let actual = self
            .input
            .read(amount)
            .map_err(|e| parser_bad_input!(e, "could not read {amount} bytes"))?;

        if amount != actual {
            return Err(parser_bad_format!(
                "attempt to read {amount} bytes, but read {actual} before reaching end of input"
            ));
        }

        Ok(())
    }

    /// Parses an UTF-8 string
    /// [name](https://webassembly.github.io/spec/core/binary/values.html#names).
    pub fn name<'b, B: crate::allocator::Buffer>(
        &mut self,
        buffer: &'b mut B,
    ) -> Result<&'b mut str> {
        let length = self.leb128_usize().context("string length")?;
        buffer.clear();
        buffer.grow(length);

        self.bytes_exact(buffer.as_mut())
            .context("string contents")?;

        #[cfg(not(feature = "alloc"))]
        let bad_encoding = |_| Error::bad_format();

        #[cfg(feature = "alloc")]
        let bad_encoding = |e| Error::bad_format().with_context(alloc::format!("{e}"));

        core::str::from_utf8_mut(buffer.as_mut()).map_err(bad_encoding)
    }
}
