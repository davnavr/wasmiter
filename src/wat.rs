//! Functions for printing section contents in the
//! [WebAssembly text format](https://webassembly.github.io/spec/core/text/index.html).

use crate::{
    index,
    parser::{self, Result as Parsed},
};
use core::fmt::{Display, Formatter};

mod instruction_text;
mod types_text;

#[must_use]
struct Writer<'a, 'b> {
    fmt: &'a mut Formatter<'b>,
    err: core::fmt::Result,
}

impl<'a, 'b> Writer<'a, 'b> {
    fn new(fmt: &'a mut Formatter<'b>) -> Self {
        Self { fmt, err: Ok(()) }
    }

    #[inline]
    fn with_fmt<F: FnOnce(&mut Formatter<'b>) -> core::fmt::Result>(&mut self, f: F) {
        self.err = self.err.and_then(|()| f(self.fmt));
    }

    fn write_char(&mut self, c: char) {
        self.with_fmt(|f| core::fmt::Write::write_char(f, c))
    }

    fn write_str(&mut self, s: &str) {
        self.with_fmt(|f| f.write_str(s))
    }

    fn write_fmt(&mut self, args: core::fmt::Arguments<'_>) {
        self.with_fmt(|f| f.write_fmt(args))
    }

    fn finish(self) -> core::fmt::Result {
        self.err
    }
}

impl<'b> core::ops::Deref for Writer<'_, 'b> {
    type Target = Formatter<'b>;

    fn deref(&self) -> &Self::Target {
        self.fmt
    }
}

fn write_err(error: &parser::Error, w: &mut Writer) {
    write!(w, "\n(;\n{error};)")
}

fn write_result<T: Display>(result: Parsed<T>, w: &mut Writer) {
    match result {
        Ok(item) => write!(w, "{item}"),
        Err(e) => write_err(&e, w),
    }
}

fn write_types<I: IntoIterator<Item = Parsed<crate::types::ValType>>>(types: I, w: &mut Writer) {
    for result in types.into_iter() {
        w.write_char(' ');
        write_result(result, w);
    }
}

macro_rules! index_format {
    ($($implementor:ty = $prefix:literal,)*) => {
        trait IndexFormat: index::Index {
            const PREFIX: char;
        }

        $(
            impl IndexFormat for $implementor {
                const PREFIX: char = $prefix;
            }
        )*
    };
}

index_format! {
    index::TypeIdx = 't',
    index::FuncIdx = 'f',
    index::TableIdx = 'T',
    index::MemIdx = 'M',
    index::GlobalIdx = 'G',
    index::ElemIdx = 'E',
    index::DataIdx = 'D',
    index::LocalIdx = 'l',
}

fn write_index<I: IndexFormat>(declaration: bool, index: I, w: &mut Writer) {
    let idx: u32 = index.into();
    if w.alternate() {
        write!(w, "${}{idx}", I::PREFIX)
    } else if declaration {
        write!(w, "(; {idx} ;)")
    } else {
        write!(w, "{idx}")
    }
}
