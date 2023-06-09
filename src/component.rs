//! Provides types and functions for parsing the various
//! [components of a WebAssembly module](https://webassembly.github.io/spec/core/syntax/modules.html)
//! from its
//! [sections in the binary format](https://webassembly.github.io/spec/core/binary/modules.html#sections).

mod code_section;
mod datas_component;
mod elems_component;
mod exports_component;
mod funcs_component;
mod function_section;
mod globals_component;
mod imports_component;
mod index_vector;
mod known_section;
mod mems_component;
mod result_type;
mod tables_component;
mod tags_component;
mod type_parser;
mod types_component;

pub use type_parser::{
    block_type, func_type, global_mutability, global_type, limits, mem_type, ref_type, table_type,
    val_type,
};

pub use code_section::{Code, CodeSection, Locals};
pub use datas_component::{DataMode, DatasComponent};
pub use elems_component::{ElementExpressions, ElementInit, ElementMode, ElemsComponent};
pub use exports_component::{Export, ExportKind, ExportsComponent};
pub use funcs_component::{Func, FuncsComponent};
pub use function_section::FunctionSection;
pub use globals_component::GlobalsComponent;
pub use imports_component::{Import, ImportKind, ImportsComponent};
pub use index_vector::IndexVector;
pub use known_section::KnownSection;
pub use mems_component::MemsComponent;
pub use result_type::ResultType;
pub use tables_component::TablesComponent;
pub use tags_component::{parse as tag, Tag, TagsComponent};
pub use types_component::TypesComponent;

/// Parses a
/// [WebAssembly index](https://webassembly.github.io/spec/core/binary/modules.html#indices).
pub fn index<N: crate::index::Index, B: crate::bytes::Bytes>(
    offset: &mut u64,
    bytes: B,
) -> crate::parser::Result<N> {
    use crate::parser::{leb128, ResultExt as _};

    let index = leb128::u32(offset, bytes).context(N::NAME)?;
    Ok(N::try_from(index)?)
}
