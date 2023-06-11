use crate::{
    bytes::Bytes,
    parser::name::{self, CharsLossy},
};
use core::fmt::{Debug, Display, Formatter, Result, Write as _};

impl<B: Bytes> CharsLossy<B> {
    fn fmt_debug(self, f: &mut Formatter<'_>) -> Result {
        f.write_char('"')?;

        for c in self {
            match c {
                '\0' => f.write_str("\\u{0}")?,
                '\r' => f.write_str("\\r")?,
                '\t' => f.write_str("\\t")?,
                '\n' => f.write_str("\\n")?,
                '\'' => f.write_str("\\'")?,
                '\"' => f.write_str("\\\"")?,
                '\\' => f.write_str("\\\\")?,
                ' '..='~' => f.write_char(c)?,
                _ => write!(f, "\\u{{{:X}}}", u32::from(c))?,
            }
        }

        f.write_char('"')
    }

    fn fmt_display(self, f: &mut Formatter<'_>) -> Result {
        for c in self {
            f.write_char(c)?;
        }
        Ok(())
    }
}

impl<B: Bytes> Debug for CharsLossy<B> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.borrowed().fmt_debug(f)
    }
}

impl<B: Bytes> Display for CharsLossy<B> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.borrowed().fmt_display(f)
    }
}

impl<B: Bytes> Debug for name::Chars<B> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        CharsLossy::new(self.borrowed()).fmt_debug(f)
    }
}

impl<B: Bytes> Display for name::Chars<B> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        CharsLossy::new(self.borrowed()).fmt_display(f)
    }
}

impl<B: Bytes> Debug for name::Name<B> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.borrowed().chars_lossy().fmt_debug(f)
    }
}

impl<B: Bytes> Display for name::Name<B> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.borrowed().chars_lossy().fmt_display(f)
    }
}
