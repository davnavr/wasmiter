use crate::input::{Input, Result, Window};
use core::{
    fmt::{Debug, Display, Formatter, LowerHex, UpperHex, Write as _},
    num::NonZeroU8,
};

/// An array of at most 16 bytes read from an [`Input`].
///
/// Returned by the [`HexDump`] iterator.
#[derive(Clone, Copy)]
pub struct Row {
    offset: u64,
    count: NonZeroU8,
    bytes: [u8; 16],
}

impl Row {
    /// The offset of into the [`Input`] to the first byte of the [`Row`]'s contents.
    #[inline]
    pub fn offset(&self) -> u64 {
        self.offset
    }

    /// Gets the contents of the row as a slice.
    #[inline]
    pub fn contents(&self) -> &[u8] {
        &self.bytes[..self.count.get().into()]
    }

    #[inline]
    fn fmt_hex(
        &self,
        f: &mut Formatter<'_>,
        mut hex: impl FnMut(u8, &mut Formatter<'_>) -> core::fmt::Result,
    ) -> core::fmt::Result {
        if f.alternate() {
            write!(f, "0x")?;
        }

        let mut first = true;
        for (offset, byte) in (self.offset..=u64::MAX).zip(self.contents().iter().copied()) {
            if !first {
                if offset % 4 == 0 {
                    f.write_char('_')?;
                }

                if offset % 8 == 0 {
                    f.write_char('_')?;
                }
            }

            first = false;
            hex(byte, f)?;
        }

        Ok(())
    }
}

impl UpperHex for Row {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.fmt_hex(f, |byte, f| write!(f, "{byte:X}"))
    }
}

impl LowerHex for Row {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.fmt_hex(f, |byte, f| write!(f, "{byte:x}"))
    }
}

impl Debug for Row {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        #[repr(transparent)]
        struct Contents<'a>(&'a Row);

        impl Debug for Contents<'_> {
            fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
                write!(f, "{:#X}", self.0)
            }
        }

        f.debug_struct("Row")
            .field("offset", &self.offset)
            .field("contents", &Contents(self))
            .finish()
    }
}

impl Display for Row {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        if f.alternate() {
            write!(f, "{:#08X} ", self.offset)
        }

        // TODO: Print contents

        Ok(())
    }
}

/// Iterates over the contents of an [`Input`], also providing a [`Debug`](core::fmt::Debug) and
/// [`Display`] implementation.
pub struct HexDump<I: Input> {
    window: Window<I>,
}

impl<I: Input> From<Window<I>> for HexDump<I> {
    #[inline]
    fn from(window: Window<I>) -> Self {
        Self { window }
    }
}

impl<I: Input> Iterator for HexDump<I> {
    type Item = Result<Row>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.window.length() == 0 {
            return None;
        }

        let mut buffer = [0u8; 16];
        let start = self.window.base();
        let result = self.window.read_at(start, &mut buffer).and_then(|buf| {
            let len = buf.len() as u8;
            self.window.advance(len.into())?;
            Ok(NonZeroU8::new(len))
        });

        let length = match result {
            Ok(Some(len)) => Ok(len),
            Ok(None) => Err(None),
            Err(e) => Err(Some(e)),
        };

        match length {
            Ok(count) => Some(Ok(Row {
                offset: start,
                count,
                bytes: buffer,
            })),
            Err(e) => {
                self.window.shrink(self.window.length());
                e.map(Err)
            }
        }
    }
}

impl<I: Input> core::fmt::Debug for HexDump<I> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        todo!("View with borrowed Window")
    }
}
