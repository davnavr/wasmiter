//! Provides types for parsing the various
//! [components of a WebAssembly module](https://webassembly.github.io/spec/core/syntax/modules.html)
//! from its
//! [sections in the binary format](https://webassembly.github.io/spec/core/binary/modules.html#sections).

mod types;
mod types_component;

pub use types::{FuncType, NumType, RefType, ValType, VecType};
pub use types_component::TypesComponent;
