//! Functions for printing section contents in the
//! [WebAssembly text format](https://webassembly.github.io/spec/core/text/index.html).

use crate::{
    parser::{self, Parsed},
    types,
};

mod datas_text;
mod display_impls;
mod elems_text;
mod exports_text;
mod funcs_text;
mod globals_text;
mod imports_text;
mod instruction_text;
mod mems_text;
mod module_text;
mod tables_text;
mod tags_text;
mod types_text;
mod writer;

use writer::Writer;

type Result<T = ()> = parser::MixedResult<T, core::fmt::Error>;

const INDENTATION: &str = "  ";

trait Wat {
    fn write(self, writer: &mut Writer) -> Result;
}

fn write_types<I: IntoIterator<Item = Parsed<types::ValType>>>(types: I, w: &mut Writer) -> Result {
    for result in types.into_iter() {
        write!(w, " {}", result?)?;
    }
    Ok(())
}

fn write_index<I: Into<u32>>(declaration: bool, index: I, w: &mut Writer) -> Result {
    let idx: u32 = index.into();
    if declaration {
        write!(w, "(; {idx} ;)")
    } else {
        write!(w, "{idx}")
    }
}

fn write_type_use(index: crate::index::TypeIdx, w: &mut Writer) -> Result {
    w.write_str("(type ")?;
    write_index(false, index, w)?;
    w.write_char(')')
}

fn write_limits(limits: &types::Limits, w: &mut Writer) -> Result {
    write!(w, "{}", limits.minimum())?;
    if let Some(maximum) = limits.maximum() {
        write!(w, " {maximum}")?;
    }
    Ok(())
}

fn write_table_type(table_type: &types::TableType, w: &mut Writer) -> Result {
    write_limits(table_type.limits(), w)?;
    write!(w, " {}", table_type.element_type())
}

fn write_mem_type(memory_type: &types::MemType, w: &mut Writer) -> Result {
    if matches!(memory_type.index_type(), types::IdxType::I64) {
        w.write_str("i64 ")?;
    }

    if matches!(memory_type.share(), types::Sharing::Shared) {
        w.write_str("(; shared ;) ")?;
    }

    write_limits(memory_type, w)
}

fn write_global_type(global_type: types::GlobalType, w: &mut Writer) -> Result {
    match global_type.mutability() {
        types::GlobalMutability::Constant => write!(w, "{}", global_type.value_type()),
        types::GlobalMutability::Variable => write!(w, "(mut {})", global_type.value_type()),
    }
}
