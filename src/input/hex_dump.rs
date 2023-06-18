use crate::input::{Input, Result, Window};
use core::{
    fmt::{Debug, Display, Formatter, LowerHex, UpperHex, Write as _},
    num::NonZeroU8,
};

/// A row of at most 16 bytes obtained from a [`HexDump`].
#[derive(Clone, Copy)]
pub struct HexDumpRow {
    offset: u64,
    count: NonZeroU8,
    bytes: [u8; 16],
}

impl HexDumpRow {
    /// The offset of into the [`Input`] to the first byte of the [`HexDumpRow`]'s contents.
    #[inline]
    pub fn offset(&self) -> u64 {
        self.offset
    }

    /// Gets the contents of the row as a slice.
    #[inline]
    pub fn contents(&self) -> &[u8] {
        &self.bytes[..self.count.get().into()]
    }

    fn bytes(&self) -> impl core::iter::FusedIterator<Item = (u64, u8)> + '_ {
        (self.offset..=u64::MAX).zip(self.contents().iter().copied())
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
        for (offset, byte) in self.bytes() {
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

    fn fmt_display(&self, f: &mut Formatter<'_>, offset_width: usize) -> core::fmt::Result {
        if f.alternate() {
            write!(f, "{:0offset_width$X}  ", self.offset - (self.offset % 16))?;
        }

        // Padding
        let skipped = (self.offset % 16) as u8;
        {
            let pad_amount = skipped * 3 + u8::from(skipped >= 8);
            for _ in 0..pad_amount {
                f.write_char(' ')?;
            }
        }

        // Bytes
        let mut first = true;
        for (offset, byte) in self.bytes() {
            if !first {
                f.write_char(' ')?;

                if offset % 16 == 8 {
                    f.write_char(' ')?;
                }
            }

            first = false;
            write!(f, "{byte:02X}")?;
        }

        if f.alternate() {
            let written = self.count.get() + skipped;
            let remaining = 16 - written;
            if remaining > 0 {
                let pad_amount = remaining * 3 + u8::from(remaining <= 8 && remaining > 0);
                for _ in 0..=pad_amount {
                    f.write_char(' ')?;
                }
            }

            f.write_str("  |")?;

            for _ in 0..skipped {
                f.write_char(' ')?;
            }

            for byte in self.contents().iter() {
                if matches!(byte, 0x20..=0x7E) {
                    if let Some(c) = char::from_u32(u32::from(*byte)) {
                        f.write_char(c)?;
                    } else {
                        return Err(core::fmt::Error);
                    }
                } else {
                    f.write_char('.')?;
                }
            }

            for _ in 0..remaining {
                f.write_char('.')?;
            }

            f.write_char('|')?;
        }

        Ok(())
    }
}

impl UpperHex for HexDumpRow {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.fmt_hex(f, |byte, f| write!(f, "{byte:X}"))
    }
}

impl LowerHex for HexDumpRow {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.fmt_hex(f, |byte, f| write!(f, "{byte:x}"))
    }
}

impl Debug for HexDumpRow {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        #[repr(transparent)]
        struct Contents<'a>(&'a HexDumpRow);

        impl Debug for Contents<'_> {
            fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
                write!(f, "{:X}", self.0)
            }
        }

        if !f.alternate() {
            f.debug_struct("Row")
                .field("offset", &self.offset)
                .field("contents", &Contents(self))
                .finish()
        } else {
            UpperHex::fmt(&self, f)
        }
    }
}

impl Display for HexDumpRow {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.fmt_display(f, 4)
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

impl<'a> From<&'a [u8]> for HexDump<&'a [u8]> {
    fn from(bytes: &'a [u8]) -> Self {
        Self::from(Window::with_offset_and_length(
            bytes,
            0,
            u64::try_from(bytes.len()).unwrap_or(u64::MAX),
        ))
    }
}

/// Borrows the underlying [`Input`].
impl<'a, I: Input> From<&'a HexDump<I>> for HexDump<&'a I> {
    fn from(borrowed: &'a HexDump<I>) -> Self {
        HexDump {
            window: (&borrowed.window).into(),
        }
    }
}

/// Clones the underlying [`Input`].
impl<I: Clone + Input> From<&HexDump<&I>> for HexDump<I> {
    fn from(borrowed: &HexDump<&I>) -> Self {
        HexDump {
            window: (&borrowed.window).into(),
        }
    }
}

impl<I: Input> Iterator for HexDump<I> {
    type Item = Result<HexDumpRow>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.window.length() == 0 {
            return None;
        }

        let mut bytes = [0u8; 16];
        let start = self.window.base();
        let fill_length = 16 - (start % 16) as usize;
        let result = self
            .window
            .read_at(start, &mut bytes[0..fill_length])
            .and_then(|buf| {
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
            Ok(count) => Some(Ok(HexDumpRow {
                offset: start,
                count,
                bytes,
            })),
            Err(e) => {
                self.window.shrink(self.window.length());
                e.map(Err)
            }
        }
    }
}

impl<I: Input> Display for HexDump<I> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        const OFFSET_HEADER: &str = "offset";

        let max_width = usize::try_from(
            self.window
                .base()
                .saturating_add(self.window.length())
                .checked_ilog(16)
                .unwrap_or(0u32),
        )
        .map(|log| log.saturating_add(1))
        .unwrap_or(4);

        let offset_width = core::cmp::max(max_width, OFFSET_HEADER.len());

        if f.alternate() {
            write!(f, "{:<offset_width$}  ", OFFSET_HEADER)?;
            writeln!(f, " 0  1  2  3  4  5  6  7   8  9  A  B  C  D  E  F")?;
        }

        let hex_dump: HexDump<&I> = self.into();
        for row in hex_dump.filter_map(Result::ok) {
            row.fmt_display(f, offset_width)?;
            writeln!(f)?;
        }

        Ok(())
    }
}

impl<I: Input> Debug for HexDump<I> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let hex_dump: HexDump<&I> = self.into();
        f.debug_list().entries(hex_dump).finish()
    }
}
