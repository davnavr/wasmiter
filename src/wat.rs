//! Functions for printing section contents in the
//! [WebAssembly text format](https://webassembly.github.io/spec/core/text/index.html).

use crate::{
    index,
    parser::{self, Result as Parsed},
    types,
};
use core::fmt::Formatter;

mod datas_text;
mod display_impls;
mod exports_text;
mod imports_text;
mod instruction_text;
mod module_text;
mod types_text;

#[must_use]
struct Writer<'a, 'b> {
    fmt: &'a mut Formatter<'b>,
    paren_count: u32,
    err: core::fmt::Result,
}

impl<'a, 'b> Writer<'a, 'b> {
    fn new(fmt: &'a mut Formatter<'b>) -> Self {
        Self {
            fmt,
            err: Ok(()),
            paren_count: 0,
        }
    }

    #[inline]
    fn with_fmt<F: FnOnce(&mut Formatter<'b>) -> core::fmt::Result>(&mut self, f: F) {
        self.err = self.err.and_then(|()| f(self.fmt));
    }

    #[inline]
    fn open_paren(&mut self) {
        self.paren_count += 1;
        self.write_char('(');
    }

    #[inline]
    fn close_paren(&mut self) {
        if let Some(updated) = self.paren_count.checked_sub(1) {
            self.paren_count = updated;
            self.write_char(')');
        }
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

    fn finish(mut self) -> core::fmt::Result {
        for _ in 0..self.paren_count {
            self.write_char(')');
        }
        self.err
    }
}

impl<'b> core::ops::Deref for Writer<'_, 'b> {
    type Target = Formatter<'b>;

    fn deref(&self) -> &Self::Target {
        self.fmt
    }
}

const INDENTATION: &str = "  ";

trait Wat {
    fn write(self, writer: &mut Writer) -> Parsed<()>;
}

fn write_err(error: &parser::Error, w: &mut Writer) {
    write!(w, "\n(;\n{error};)")
}

fn write_types<I: IntoIterator<Item = Parsed<types::ValType>>>(
    types: I,
    w: &mut Writer,
) -> Parsed<()> {
    for result in types.into_iter() {
        write!(w, " {}", result?);
    }
    Ok(())
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

fn write_type_use(index: crate::index::TypeIdx, w: &mut Writer) {
    w.write_str("(type ");
    write_index(false, index, w);
    w.write_char(')');
}

fn write_limits(limits: &types::Limits, w: &mut Writer) {
    write!(w, "{}", limits.minimum());
    if let Some(maximum) = limits.maximum() {
        write!(w, " {maximum}");
    }
}

fn write_table_type(table_type: &types::TableType, w: &mut Writer) {
    write_limits(table_type.limits(), w);
    write!(w, " {}", table_type.element_type());
}

fn write_mem_type(memory_type: &types::MemType, w: &mut Writer) {
    if matches!(memory_type.index_type(), types::IdxType::I64) {
        w.write_str("i64 ");
    }

    if matches!(memory_type.share(), types::Sharing::Shared) {
        w.write_str("(; shared ;) ");
    }

    write_limits(memory_type, w)
}

fn write_global_type(global_type: types::GlobalType, w: &mut Writer) {
    match global_type.mutability() {
        types::GlobalMutability::Constant => write!(w, "{}", global_type.value_type()),
        types::GlobalMutability::Variable => write!(w, "(mut {})", global_type.value_type()),
    }
}
