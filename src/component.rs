//! Provides types for parsing the various
//! [components of a WebAssembly module](https://webassembly.github.io/spec/core/syntax/modules.html)
//! from its
//! [sections in the binary format](https://webassembly.github.io/spec/core/binary/modules.html#sections).

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
    DataIdx, ElemIdx, FuncIdx, GlobalIdx, LabelIdx, LocalIdx, MemIdx, TableIdx, TypeIdx,
};

pub use types::{
    BlockType, GlobalMutability, GlobalType, NumType, RefType, TableType, ValType, VecType,
};

pub use function_section::FunctionSection;
pub use globals_component::{Global, GlobalsComponent};
pub use imports_component::{Import, ImportKind, ImportParser, ImportsComponent};
pub use known_section::KnownSection;
pub use limits::{Limits, MemType};
pub use mems_component::MemsComponent;
pub use tables_component::TablesComponent;
pub use types_component::{ResultType, TypesComponent};

pub(crate) fn debug_section_contents<T: core::fmt::Debug>(
    iterator: crate::parser::Result<impl core::iter::Iterator<Item = crate::parser::Result<T>>>,
    f: &mut core::fmt::Formatter,
) -> core::fmt::Result {
    let mut list = f.debug_list();
    match iterator {
        Ok(items) => list.entries(items),
        Err(e) => list.entries(core::iter::once(crate::parser::Result::<()>::Err(e))),
    }
    .finish()
}
