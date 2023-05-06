//! Provides types for parsing the various
//! [components of a WebAssembly module](https://webassembly.github.io/spec/core/syntax/modules.html)
//! from its
//! [sections in the binary format](https://webassembly.github.io/spec/core/binary/modules.html#sections).

mod function_section;
mod imports_component;
mod index;
mod known_section;
mod limits;
mod tables_component;
mod type_parser;
mod types;
mod types_component;

pub use index::{
    DataIdx, ElemIdx, FuncIdx, GlobalIdx, LabelIdx, LocalIdx, MemIdx, TableIdx, TypeIdx,
};

pub use types::{
    BlockType, FuncType, GlobalMutability, GlobalType, NumType, RefType, TableType, ValType,
    VecType,
};

pub use function_section::FunctionSection;
pub use imports_component::{Import, ImportKind, ImportsComponent};
pub use known_section::KnownSection;
pub use limits::{Limits, MemType};
pub use tables_component::TablesComponent;
pub use types_component::TypesComponent;
