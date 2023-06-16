use crate::{
    input::Input,
    parser::name::{self, CharsLossy},
};
use core::fmt::{Debug, Display, Formatter, Result, Write as _};

impl<I: Input> CharsLossy<I> {
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

impl<I: Input> Debug for CharsLossy<I> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.borrowed().fmt_debug(f)
    }
}

impl<I: Input> Display for CharsLossy<I> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.borrowed().fmt_display(f)
    }
}

impl<I: Input> Debug for name::Chars<I> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        CharsLossy::new(self.borrowed()).fmt_debug(f)
    }
}

impl<I: Input> Display for name::Chars<I> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        CharsLossy::new(self.borrowed()).fmt_display(f)
    }
}

impl<I: Input> Debug for name::Name<I> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.borrowed().chars_lossy().fmt_debug(f)
    }
}

impl<I: Input> Display for name::Name<I> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.borrowed().chars_lossy().fmt_display(f)
    }
}
