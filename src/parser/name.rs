//! Types and functions for parsing UTF-8 strings from [`Input`] bytes.

use crate::{
    input::{self, BorrowInput, Input, Window},
    parser::{self, Error, ResultExt as _},
};

mod char_iterators;
mod error;
mod name_fmt;

pub use char_iterators::{Chars, CharsLossy};
pub use error::{InvalidCodePoint, NameError};

/// A UTF-8 string [name](https://webassembly.github.io/spec/core/binary/values.html#names).
#[derive(Clone, Copy)]
pub struct Name<I: Input> {
    input: I,
    offset: u64,
    length: u32,
}

impl<I: Input> Name<I> {
    /// Reads a length-prefixed UTF-8 string from the given [`Input`], starting at the given
    /// `offset`.
    ///
    /// # Error
    ///
    /// Returns an error if the length could not be parsed.
    pub fn new(input: I, offset: &mut u64) -> parser::Parsed<Self> {
        Ok(Self {
            length: parser::leb128::u32(offset, &input).context("string length")?,
            offset: *offset,
            input,
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

    /// Returns an iterator over the [`char`]s of the [`Name`], returning a [`NameError`] for
    /// invalid code points or failures retrieving bytes from the [`Input`].
    #[inline]
    pub fn chars(self) -> Chars<I> {
        Chars::new(self)
    }

    /// Returns an iterator over the [`char`]s of the [`Name`], emitting a
    /// [`char::REPLACEMENT_CHARACTER`] for each invalid code point or failure to retrieve
    /// bytes from the [`Input`].
    #[inline]
    pub fn chars_lossy(self) -> CharsLossy<I> {
        CharsLossy::new(Chars::new(self))
    }

    /// Copies the contents of the [`Name`] into the specified `buffer`.
    ///
    /// If the length of the `buffer` is less than the length, in bytes, of the [`Name`], then only
    /// a portion of the [`Name`] contents is copied.
    ///
    /// # Errors
    ///
    /// Returns an error if the name bytes could not be fetched from the [`Input`].
    pub fn copy_to_slice<'b>(&self, buffer: &'b mut [u8]) -> parser::Parsed<&'b mut [u8]> {
        let length = core::cmp::min(
            usize::try_from(self.length).ok().unwrap_or(usize::MAX),
            buffer.len(),
        );

        let destination: &'b mut [u8] = &mut buffer[0..length];

        self.input
            .read_exact_at(self.offset, destination)
            .context("string contents")?;

        Ok(destination)
    }

    /// Returns the contents of the [`Name`] as a [`Window`](input::Window).
    pub fn into_bytes_window(self) -> input::Window<I> {
        let offset = self.offset;
        let length = self.length();
        input::Window::with_offset_and_length(self.input, offset, length)
    }

    /// Attempts to compare this [`Name`] to a [`str`]ing, returning `true` if they are equal.
    ///
    /// # Errors
    ///
    /// Returns an error if the name bytes could not be feteched from the [`Input`].
    #[inline]
    pub fn try_eq_str(&self, s: &str) -> parser::Parsed<bool> {
        self.borrow_input()
            .into_bytes_window()
            .try_eq_at(self.offset, s.as_bytes())
            .map_err(Into::into)
    }
}

impl<I: Input> input::HasInput<I> for Name<I> {
    #[inline]
    fn input(&self) -> &I {
        &self.input
    }
}

impl<'a, I: Input + 'a> BorrowInput<'a, I> for Name<I> {
    type Borrowed = Name<&'a I>;

    #[inline]
    fn borrow_input(&'a self) -> Name<&'a I> {
        Name {
            input: &self.input,
            offset: self.offset,
            length: self.length,
        }
    }
}

impl<'a, I: Clone + Input + 'a> input::CloneInput<'a, I> for Name<&'a I> {
    type Cloned = Name<I>;

    #[inline]
    fn clone_input(&self) -> Name<I> {
        Name {
            input: self.input.clone(),
            offset: self.offset,
            length: self.length,
        }
    }
}

impl<I: Input> From<Name<I>> for Window<I> {
    /// Converts a [`Name`] into a [`Window`] by calling [`Name::into_bytes_window`].
    #[inline]
    fn from(name: Name<I>) -> Self {
        name.into_bytes_window()
    }
}

impl<I: Input> From<Name<input::Window<I>>> for Name<I> {
    fn from(name: Name<input::Window<I>>) -> Name<I> {
        let mut length = core::cmp::min(
            name.length,
            u32::try_from(name.input.length()).unwrap_or(u32::MAX),
        );

        if name.offset > name.input.base() + name.input.length() {
            length = 0;
        }

        Name {
            offset: name.offset,
            length,
            input: name.input.into_inner(),
        }
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
impl<I: Input> Name<I> {
    /// Allocates a byte vector to contain the contents of the [`Name`].
    ///
    /// # Error
    ///
    /// Returns an error if the operation to read the characters from the [`Input`] fails.
    pub fn into_bytes(self) -> parser::Parsed<alloc::vec::Vec<u8>> {
        let mut bytes = alloc::vec![0u8; self.length.try_into().unwrap_or(usize::MAX)];
        self.copy_to_slice(&mut bytes)?;
        Ok(bytes)
    }

    /// Allocates a [`String`](alloc::string::String) containing the contents of the [`Name`].
    ///
    /// # Error
    ///
    /// Returns an error if the operation to read the characters from the [`Input`] fails, or if
    /// the [`Name`] is not valid UTF-8.
    pub fn try_into_string(self) -> parser::Parsed<alloc::string::String> {
        alloc::string::String::from_utf8(self.into_bytes()?).map_err(Into::into)
    }
}

/// Parses a UTF-8 string [`Name`].
pub fn parse<I: Input>(offset: &mut u64, input: I) -> parser::Parsed<Name<I>> {
    let name = Name::new(input, offset)?;
    input::increment_offset(offset, name.length() as usize)?;
    Ok(name)
}

impl<'a> TryFrom<&'a [u8]> for Name<&'a [u8]> {
    type Error = Error;

    fn try_from(bytes: &'a [u8]) -> parser::Parsed<Self> {
        let actual_length = bytes.len();
        if let Ok(length) = u32::try_from(actual_length) {
            Ok(Self {
                length,
                input: bytes,
                offset: 0,
            })
        } else {
            #[cold]
            #[inline(never)]
            fn slice_too_large(length: usize) -> Error {
                Error::new(parser::ErrorKind::InvalidFormat).with_context(
                    parser::Context::from_closure(move |f| {
                        write!(f, "byte slice has a length of {length}, which is too large")
                    }),
                )
            }

            Err(slice_too_large(actual_length))
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
    type Error = Error;

    fn try_from(s: &'a str) -> parser::Parsed<Self> {
        Self::try_from(s.as_bytes())
    }
}

impl<I: Input> IntoIterator for Name<I> {
    type IntoIter = Chars<I>;
    type Item = Result<char, NameError>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.chars()
    }
}
