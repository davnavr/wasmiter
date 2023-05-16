//! Provides types and functions for parsing the various
//! [components of a WebAssembly module](https://webassembly.github.io/spec/core/syntax/modules.html)
//! from its
//! [sections in the binary format](https://webassembly.github.io/spec/core/binary/modules.html#sections).

mod elems_component;
mod exports_component;
mod function_section;
mod globals_component;
mod imports_component;
mod index;
mod known_section;
mod limits;
mod mems_component;
mod tables_component;
mod type_parser;
mod types;
mod types_component;

pub use index::{
    index, DataIdx, ElemIdx, FuncIdx, GlobalIdx, LabelIdx, LocalIdx, MemIdx, TableIdx, TypeIdx,
};

pub use types::{
    BlockType, GlobalMutability, GlobalType, NumType, RefType, TableType, ValType, VecType,
};

pub use type_parser::{
    block_type, func_type, global_mutability, global_type, limits, mem_type, ref_type, table_type,
    val_type,
};

pub use elems_component::{ElemsComponent, ElementExpressions, ElementInit, ElementKind};
pub use exports_component::{Export, ExportKind, ExportsComponent};
pub use function_section::FunctionSection;
pub use globals_component::GlobalsComponent;
pub use imports_component::{Import, ImportKind, ImportsComponent};
pub use known_section::KnownSection;
pub use limits::{Limits, MemType};
pub use mems_component::MemsComponent;
pub use tables_component::TablesComponent;
pub use types_component::{ResultType, TypesComponent};
