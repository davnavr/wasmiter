use core::fmt::{Display, Formatter, Write as _};

/// Describes an invalid byte sequence or code point that was encountered while decoding a
/// [`Name`](crate::parser::name::Name).
#[derive(Clone, Copy)]
pub struct InvalidCodePoint {
    pub(super) length: core::num::NonZeroU8,
    pub(super) bytes: [u8; 4],
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

impl core::fmt::Debug for InvalidCodePoint {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        if let Some(bytes) = self.bytes() {
            core::fmt::Debug::fmt(bytes, f)
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

/// Errors which can occur when attempting to interpret a [`Name`](crate::parser::name::Name) as a
/// UTF-8 string.
#[derive(Debug)]
pub enum NameError {
    /// An operation to read the UTF-8 string contents from the [`Bytes`](crate::bytes::Bytes)
    /// failed.
    BadInput(crate::bytes::Error),
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
