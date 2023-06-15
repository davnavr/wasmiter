use core::fmt::{Display, Formatter};

#[derive(Clone, Copy, Debug)]
pub(super) enum ErrorKind {
    OutOfBounds,
    CannotFillBuffer,
    OffsetOverflow,
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::OutOfBounds => "operation would be out of bounds",
            Self::CannotFillBuffer => "buffer could not be completely filled",
            Self::OffsetOverflow => "offset would overflow",
        })
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "alloc")] {
        struct BoxedError {
            kind: ErrorKind,
            offset: u64,
            length: Option<u64>
        }

        type ErrorInner = alloc::boxed::Box<BoxedError>;
    } else {
        enum InlineError {
            NoOffset(ErrorKind),
            WithOffset {
                kind: ErrorKind,
                offset: u32,
            },
        }

        type ErrorInner = InlineError;
    }
}

/// Error type used when an operation to read [`Input`](crate::input::Input) fails.
#[repr(transparent)]
pub struct Error {
    inner: ErrorInner,
}

impl Error {
    #[must_use]
    pub(super) fn new(kind: ErrorKind, offset: u64, length: Option<u64>) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(feature = "alloc")] {
                Self {
                    inner: alloc::boxed::Box::new(BoxedError { kind, offset, length }),
                }
            } else {
                let _ = length;
                Self {
                    inner: if let Ok(small_offset) = u32::try_from(offset) {
                        InlineError::WithOffset { kind, offset: small_offset }
                    } else {
                        InlineError::NoOffset(kind)
                    },
                }
            }
        }
    }

    fn kind(&self) -> ErrorKind {
        cfg_if::cfg_if! {
            if #[cfg(feature = "alloc")] {
                self.inner.kind
            } else {
                let (InlineError::WithOffset { kind, .. } | InlineError::NoOffset(kind)) = self.inner;
                kind
            }
        }
    }
}

#[cfg(feature = "alloc")]
impl Error {
    /// Gets the offset from the start of the [`Input`](crate::input::Input) where the error
    /// occured.
    #[inline]
    pub fn offset(&self) -> u64 {
        self.inner.offset
    }

    /// Gets the remaining length of the [`Input`](crate::input::Input) where the error occured.
    #[inline]
    pub fn length(&self) -> Option<u64> {
        self.inner.length
    }
}

impl core::fmt::Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut s = f.debug_struct("Error");
        s.field("kind", &self.kind());

        cfg_if::cfg_if! {
            if #[cfg(feature = "alloc")] {
                s.field("offset", &self.offset()).field("length", &self.length());
            } else {
                if let InlineError::WithOffset { offset, .. } = self.inner {
                    s.field("offset", &offset);
                }
            }
        }

        s.finish()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.kind())?;

        cfg_if::cfg_if! {
            if #[cfg(feature = "alloc")] {
                write!(f, " at offset {}", self.offset())?;

                if let Some(length) = self.length() {
                    write!(f, ", length of input is {}", length)?;
                }
            } else {
                if let InlineError::WithOffset { offset, .. } = self.inner {
                    write!(f, " at offset {}", offset)?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}
