//! Functions for printing section contents in the
//! [WebAssembly text format](https://webassembly.github.io/spec/core/text/index.html).

use crate::{
    bytes::Bytes,
    component,
    parser::{self, Result as Parsed},
    types::ValType,
};
use core::fmt::{Display, Formatter};

type Result<T = ()> = core::result::Result<T, core::fmt::Error>;

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

fn write_types<I: IntoIterator<Item = Parsed<ValType>>>(types: I, w: &mut Writer) {
    for result in types.into_iter() {
        w.write_char(' ');
        write_result(result, w);
    }
}

fn write_index(prefix: char, declaration: bool, index: u32, w: &mut Writer) {
    if w.alternate() {
        write!(w, "${prefix}{index}")
    } else if declaration {
        write!(w, "(; {index} ;)")
    } else {
        write!(w, "{index}")
    }
}

impl<B: Bytes> Display for component::TypesComponent<B> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut types = self.borrowed();
        let mut w = Writer::new(f);

        for i in 0u32.. {
            let result = types.parse(
                |params| Ok(params.dereferenced()),
                |params, results| {
                    w.write_str("(type ");
                    write_index('t', true, i, &mut w);
                    w.write_str(" (func (param");
                    write_types(params, &mut w);
                    w.write_str(") (result");
                    write_types(results, &mut w);
                    w.write_str("))");
                    Ok(())
                },
            );

            if let Err(e) = &result {
                write_err(e, &mut w);
            }

            if let Ok(Some(())) = result {
                writeln!(w, ")");
                continue;
            }

            break;
        }

        w.finish()
    }
}
