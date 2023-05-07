//! Model of the
//! [WebAssembly instruction set](https://webassembly.github.io/spec/core/syntax/instructions.html).

mod opcode;

pub use opcode::{InvalidOpcode, Opcode};

//pub struct Expression<I> // parser that keeps track of nesting state
