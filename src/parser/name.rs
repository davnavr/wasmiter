//! Types and functions for parsing UTF-8 strings from [`Bytes`].

use crate::{
    buffer::Buffer,
    bytes::{self, Bytes},
    parser::{self, ResultExt as _},
};
use core::{
    fmt::{Debug, Display, Formatter, Write},
    num::NonZeroU8,
};

/// Describes an invalid byte sequence or code point that was encountered while decoding a
/// [`Name`].
#[derive(Clone, Copy)]
pub struct InvalidCodePoint {
    length: NonZeroU8,
    bytes: [u8; 4],
}

impl InvalidCodePoint {
    fn bytes(&self) -> Option<&[u8]> {
        let length = usize::from(self.length.get());
        if length <= self.bytes.len() {
            Some(&self.bytes[0..length])
        } else {
            None
        }
    }
}

impl Debug for InvalidCodePoint {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        if let Some(bytes) = self.bytes() {
            Debug::fmt(bytes, f)
        } else {
            f.debug_struct("InvalidCodePoint")
                .field("length", &self.length)
                .finish()
        }
    }
}

impl Display for InvalidCodePoint {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str("encountered invalid byte sequence or invalid code point")?;
        if let Some(bytes) = self.bytes() {
            f.write_char(':')?;
            for b in bytes {
                write!(f, " {b:#04X}")?;
            }
            Ok(())
        } else {
            write!(f, " of length {}", self.length)
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for InvalidCodePoint {}

/// Errors which can occur when attempting to interpret a [`Name`] as a UTF-8 string.
#[derive(Debug)]
pub enum NameError {
    /// An operation to read the UTF-8 string contents from the [`Bytes`] failed.
    BadInput(bytes::Error),
    /// The UTF-8 string itself is malformed.
    BadBytes(InvalidCodePoint),
}

impl Display for NameError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::BadInput(bad) => Display::fmt(bad, f),
            Self::BadBytes(e) => Display::fmt(e, f),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for NameError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::BadInput(bad) => Some(bad),
            Self::BadBytes(bad) => Some(bad),
        }
    }
}

#[derive(Clone, Copy, Default)]
struct CharsBuffer {
    /// - Lower 4 bits contain number of saved bytes in `buffer`.
    /// - Upper 4 bits contain number of valid UTF-8 bytes in `buffer`.
    lengths: u8,
    /// Length of the error after `valid_len`.
    bad_sequence: Option<NonZeroU8>,
    /// # Safety
    ///
    /// The first `valid_len` bytes **must** be valid UTF-8.
    buffer: [u8; 15],
}

impl CharsBuffer {
    #[inline]
    fn saved_len(&self) -> u8 {
        self.lengths & 0xF
    }

    #[inline]
    fn valid_len(&self) -> u8 {
        self.lengths >> 4
    }

    fn valid(&self) -> &str {
        let valid_bytes = &self.buffer[0..usize::from(self.valid_len())];
        if cfg!(debug_assertions) {
            core::str::from_utf8(valid_bytes).unwrap()
        } else {
            // Safety: it is an invariant that valid_len bytes are valid
            unsafe { core::str::from_utf8_unchecked(valid_bytes) }
        }
    }

    fn advance(&mut self, amount: u8) {
        debug_assert!(amount <= self.saved_len());

        self.lengths = (self.lengths & 0xF0) | ((self.saved_len() - amount) & 0xF);

        if self.valid_len() > 0 {
            debug_assert!(amount <= self.valid_len());
            self.lengths = ((self.valid_len() - amount) << 4) | (self.lengths & 0xF);
        }

        let start = usize::from(amount);
        let new_length = start + usize::from(self.saved_len());
        self.buffer.copy_within(start..new_length, 0);
    }

    fn take_char(&mut self) -> Result<Option<char>, NameError> {
        if self.valid_len() > 0 {
            let mut chars = self.valid().chars();
            let original_len = chars.as_str().len();

            let c = if cfg!(debug_assertions) {
                chars.next().unwrap()
            } else {
                // Safety: check above ensures string is not empty
                unsafe { chars.next().unwrap_unchecked() }
            };

            // Skip the amount of bytes that were read
            self.advance((original_len - chars.as_str().len()) as u8);

            Ok(Some(c))
        } else if let Some(bad) = self.bad_sequence.take() {
            let bad_len = usize::from(bad.get());
            let mut bytes = [0u8; 4];
            bytes[0..bad_len].copy_from_slice(&self.buffer[0..bad_len]);

            // Skip the invalid sequence
            self.advance(bad.get());

            Err(NameError::BadBytes(InvalidCodePoint { length: bad, bytes }))
        } else {
            Ok(None)
        }
    }

    fn fill(
        &mut self,
        offset: &mut u64,
        length: &mut u32,
        bytes: &impl Bytes,
    ) -> Result<(), NameError> {
        // take_char would return Ok(Some) or Err for the same conditions here
        if self.valid_len() > 0 || self.bad_sequence.is_some() {
            return Ok(());
        }

        let saved_length = usize::from(self.saved_len());
        if saved_length < self.buffer.len() {
            let result = bytes
                .read(offset, &mut self.buffer[saved_length..])
                .map(|buf| buf.len() as u8);

            match result {
                Ok(filled) => {
                    self.lengths = (self.lengths & 0xF0) | ((saved_length as u8 + filled) & 0xF);
                    *length -= u32::from(filled);
                }
                Err(e) => {
                    *length = 0;
                    self.lengths = 0;
                    self.bad_sequence = None;
                    return Err(NameError::BadInput(e));
                }
            }
        }

        debug_assert_eq!(self.valid_len(), 0);

        let valid_len = match core::str::from_utf8(&self.buffer[0..usize::from(self.saved_len())]) {
            Ok(s) => s.len(),
            Err(e) => {
                if let Some(bad_len) = e.error_len() {
                    self.bad_sequence =
                        Some(u8::try_from(bad_len).ok().and_then(NonZeroU8::new).unwrap());
                }

                e.valid_up_to()
            }
        };

        self.lengths = ((valid_len as u8) << 4) | (self.lengths & 0xF);
        Ok(())
    }
}

/// An iterator over the [`char`]s of a [`Name`].
///
/// See the documentation for [`Name::chars()`] for more information.
#[derive(Clone, Copy)]
#[must_use]
pub struct Chars<B: Bytes> {
    name: Name<B>,
    buffer: CharsBuffer,
}

impl<B: Bytes> Chars<B> {
    fn new(name: Name<B>) -> Self {
        Self {
            name,
            buffer: Default::default(),
        }
    }

    fn borrowed(&self) -> Chars<&B> {
        Chars {
            name: self.name.borrowed(),
            buffer: self.buffer,
        }
    }

    fn next_inner(&mut self) -> Result<Option<char>, NameError> {
        if self.name.length == 0 && self.buffer.saved_len() == 0 {
            return Ok(None);
        }

        self.buffer.fill(
            &mut self.name.offset,
            &mut self.name.length,
            &self.name.bytes,
        )?;

        self.buffer.take_char()
    }
}

impl<B: Bytes> Iterator for Chars<B> {
    type Item = Result<char, NameError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.next_inner().transpose()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let max = usize::from(self.buffer.saved_len()) + (self.name.length as usize);
        (core::cmp::min(1, max), Some(max))
    }
}

impl<B: Bytes> core::iter::FusedIterator for Chars<B> {}

/// An iterator over the [`char`]s of a [`Name`] that substitutes invalid byte sequences and other errors with [`char::REPLACEMENT_CHARACTER`].
///
/// See the documentation for [`Name::chars_lossy()`] for more information.
#[derive(Clone, Copy)]
#[must_use]
pub struct CharsLossy<B: Bytes> {
    inner: Chars<B>,
}

impl<B: Bytes> CharsLossy<B> {
    #[inline]
    fn new(inner: Chars<B>) -> CharsLossy<B> {
        Self { inner }
    }

    fn borrowed(&self) -> CharsLossy<&B> {
        CharsLossy::new(self.inner.borrowed())
    }

    fn fmt_debug(self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_char('\'')?;

        for c in self {
            if c.is_ascii_graphic() || matches!(c, ' ' | char::REPLACEMENT_CHARACTER) {
                f.write_char(c)?;
            } else {
                match c {
                    '\0' => f.write_str("\\0")?,
                    '\r' => f.write_str("\\r")?,
                    '\t' => f.write_str("\\t")?,
                    '\n' => f.write_str("\\n")?,
                    '\'' => f.write_str("\\'")?,
                    '\"' => f.write_str("\\\"")?,
                    '\\' => f.write_str("\\\\")?,
                    _ => write!(f, "\\u{{{:#X}}}", u32::from(c))?,
                }
            }
        }

        f.write_char('\'')
    }

    fn fmt_display(self, f: &mut Formatter<'_>) -> core::fmt::Result {
        for c in self {
            f.write_char(c)?;
        }
        Ok(())
    }
}

impl<B: Bytes> Iterator for CharsLossy<B> {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        Some(match self.inner.next()? {
            Ok(c) => c,
            Err(_) => char::REPLACEMENT_CHARACTER,
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<B: Bytes> core::iter::FusedIterator for CharsLossy<B> {}

/// A UTF-8 string [name](https://webassembly.github.io/spec/core/binary/values.html#names).
#[derive(Clone, Copy)]
pub struct Name<B: Bytes> {
    bytes: B,
    offset: u64,
    length: u32,
}

impl<B: Bytes> Name<B> {
    /// Reads a length-prefixed UTF-8 string from the given [`Bytes`], starting at the given
    /// `offset`.
    ///
    /// # Error
    ///
    /// Returns an error if the length could not be parsed.
    pub fn new(bytes: B, mut offset: u64) -> parser::Result<Self> {
        Ok(Self {
            length: parser::leb128::u32(&mut offset, &bytes).context("string length")?,
            offset,
            bytes,
        })
    }

    /// Gets the offset to the first byte of the UTF-8 string contents.
    #[inline]
    pub fn offset(&self) -> u64 {
        self.offset
    }

    /// Gets the length, in bytes, of the UTF-8 string contents.
    #[inline]
    pub fn length(&self) -> u64 {
        self.length.into()
    }

    /// Allocates the UTF-8 string contents into the given [`Buffer`].
    pub fn to_str_in_buffer<'b, U: Buffer>(
        &self,
        buffer: &'b mut U,
    ) -> parser::Result<&'b mut str> {
        let length = usize::try_from(self.length).unwrap_or(usize::MAX);
        let start = buffer.as_mut().len();
        buffer.grow(length);
        let destination = &mut buffer.as_mut()[start..][..length];
        let mut offset = self.offset;
        parser::bytes_exact(&mut offset, &self.bytes, destination).context("string contents")?;
        core::str::from_utf8_mut(destination).map_err(|e| crate::parser_bad_format!("{e}"))
    }

    /// Returns an iterator over the [`char`]s of the [`Name`], returning a [`NameError`] for
    /// invalid code points or failures retrieving [`Bytes`].
    #[inline]
    pub fn chars(self) -> Chars<B> {
        Chars::new(self)
    }

    /// Returns an iterator over the [`char`]s of the [`Name`], emitting a
    /// [`char::REPLACEMENT_CHARACTER`] for each invalid code point or failure to retrieve
    /// [`Bytes`].
    #[inline]
    pub fn chars_lossy(self) -> CharsLossy<B> {
        CharsLossy::new(Chars::new(self))
    }

    /// Borrows the underlying [`Bytes`] to create a copy of the [`Name`].
    pub fn borrowed(&self) -> Name<&B> {
        Name {
            bytes: &self.bytes,
            offset: self.offset,
            length: self.length,
        }
    }
}

impl<B: Bytes + Clone> Name<&B> {
    pub(crate) fn really_cloned(&self) -> Name<B> {
        Name {
            bytes: self.bytes.clone(),
            offset: self.offset,
            length: self.length,
        }
    }
}

#[cfg(feature = "alloc")]
impl<B: Bytes> Name<B> {
    /// Allocates a byte vector to contain the contents of the [`Name`].
    ///
    /// # Error
    ///
    /// Returns an error if the operation to read the characters from the [`Bytes`] fails.
    pub fn into_bytes(self) -> parser::Result<Vec<u8>> {
        let mut bytes = vec![0u8; self.length.try_into().unwrap_or(usize::MAX)];
        let mut offset = self.offset;
        parser::bytes_exact(&mut offset, &self.bytes, &mut bytes).context("string contents")?;
        Ok(bytes)
    }

    /// Allocates a [`String`] containing the contents of the [`Name`].
    ///
    /// # Error
    ///
    /// Returns an error if the operation to read the characters from the [`Bytes`] fails, or if
    /// the [`Name`] is not valid UTF-8.
    pub fn try_into_string(self) -> parser::Result<alloc::string::String> {
        alloc::string::String::from_utf8(self.into_bytes()?)
            .map_err(|e| crate::parser_bad_format!("{e}"))
    }
}

/// Parses a UTF-8 string [`Name`].
pub fn parse<B: Bytes>(offset: &mut u64, bytes: B) -> parser::Result<Name<B>> {
    let name = Name::new(bytes, *offset)?;
    bytes::increment_offset(offset, name.length() as usize);
    Ok(name)
}

impl<'a> TryFrom<&'a [u8]> for Name<&'a [u8]> {
    type Error = parser::Error;

    fn try_from(bytes: &'a [u8]) -> parser::Result<Self> {
        let actual_length = bytes.len();
        if let Ok(length) = u32::try_from(actual_length) {
            Ok(Self {
                length,
                bytes,
                offset: 0,
            })
        } else {
            Err(crate::parser_bad_format!(
                "byte slice has a length of {actual_length}, which is too large"
            ))
        }
    }
}

/// Interprets the `str` as a [`Name`] originating from a byte slice.
///
/// # Examples
///
/// ```
/// # fn main() -> Result<(), wasmiter::parser::Error> {
/// use wasmiter::parser::name::Name;
/// let s = "hello";
/// let n = Name::try_from(s)?;
/// assert_eq!(format!("{n}"), s);
/// # Ok(())
/// # }
/// ```
impl<'a> TryFrom<&'a str> for Name<&'a [u8]> {
    type Error = parser::Error;

    fn try_from(s: &'a str) -> parser::Result<Self> {
        Self::try_from(s.as_bytes())
    }
}

impl<B: Bytes> Debug for CharsLossy<B> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.borrowed().fmt_debug(f)
    }
}

impl<B: Bytes> Display for CharsLossy<B> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.borrowed().fmt_display(f)
    }
}

impl<B: Bytes> Debug for Chars<B> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        CharsLossy::new(self.borrowed()).fmt_debug(f)
    }
}

impl<B: Bytes> Display for Chars<B> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        CharsLossy::new(self.borrowed()).fmt_display(f)
    }
}

impl<B: Bytes> Debug for Name<B> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.borrowed().chars_lossy().fmt_debug(f)
    }
}

impl<B: Bytes> Display for Name<B> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.borrowed().chars_lossy().fmt_display(f)
    }
}

impl<B: Bytes> IntoIterator for Name<B> {
    type IntoIter = Chars<B>;
    type Item = Result<char, NameError>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.chars()
    }
}
