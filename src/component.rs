//! Provides types for parsing the various
//! [components of a WebAssembly module](https://webassembly.github.io/spec/core/syntax/modules.html)
//! from its
//! [sections in the binary format](https://webassembly.github.io/spec/core/binary/modules.html#sections).

mod imports_component;
mod index;
mod types;
mod types_component;

pub use imports_component::{Import, ImportKind, ImportsComponent};
pub use index::{
    DataIdx, ElemIdx, FuncIdx, GlobalIdx, Index, LabelIdx, LocalIdx, MemIdx, TableIdx, TypeIdx,
};
pub use types::{FuncType, NumType, RefType, ValType, VecType};
pub use types_component::TypesComponent;
