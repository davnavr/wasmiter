use crate::{parser::MixedError, wat::Result};
use core::fmt::{Error as FmtError, Formatter};

#[must_use]
pub(super) struct Writer<'a, 'b> {
    fmt: &'a mut Formatter<'b>,
    paren_count: u32,
}

impl<'a, 'b> Writer<'a, 'b> {
    pub(super) fn new(fmt: &'a mut Formatter<'b>) -> Self {
        Self {
            fmt,
            paren_count: 0,
        }
    }

    #[inline]
    fn with_fmt<F: FnOnce(&mut Formatter<'b>) -> core::fmt::Result>(&mut self, f: F) -> Result {
        f(self.fmt).map_err(MixedError::User)
    }

    #[inline]
    pub(super) fn open_paren(&mut self) -> Result {
        self.paren_count = self
            .paren_count
            .checked_add(1)
            .ok_or(MixedError::User(FmtError))?;
        self.write_char('(')
    }

    #[inline]
    pub(super) fn close_paren(&mut self) -> Result {
        self.paren_count = self
            .paren_count
            .checked_sub(1)
            .ok_or(MixedError::User(FmtError))?;
        self.write_char(')')
    }

    pub(super) fn write_char(&mut self, c: char) -> Result {
        self.with_fmt(|f| core::fmt::Write::write_char(f, c))
    }

    pub(super) fn write_str(&mut self, s: &str) -> Result {
        self.with_fmt(|f| f.write_str(s))
    }

    pub(super) fn write_fmt(&mut self, args: core::fmt::Arguments<'_>) -> Result {
        self.with_fmt(|f| f.write_fmt(args))
    }

    pub(super) fn finish(mut self, result: Result) -> core::fmt::Result {
        if let Err(MixedError::Parser(error)) = &result {
            write!(self.fmt, "\n(;\n{error};)\n")?;
        }

        for _ in 0..self.paren_count {
            <_ as core::fmt::Write>::write_char(&mut self.fmt, ')')?;
        }

        match result {
            Ok(()) => Ok(()),
            Err(MixedError::User(err)) => Err(err),
            Err(MixedError::Parser(_)) => Err(FmtError),
        }
    }
}
