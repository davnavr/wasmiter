//! Model of the
//! [WebAssembly instruction set](https://webassembly.github.io/spec/core/syntax/instructions.html).

mod instruction;
mod instruction_sequence;
mod is_constant;
mod memarg;
mod opcode;
mod prefix_fc;
mod vector_opcode;

#[doc(no_inline)]
pub use crate::component::{BlockType, LabelIdx, LocalIdx};
pub use instruction::{Instruction, LaneIdx};
pub use instruction_sequence::InstructionSequence;
pub use is_constant::IsConstant;
pub use memarg::{Align, MemArg};
pub use opcode::{InvalidOpcode, Opcode};
pub use prefix_fc::FCPrefixedOpcode;
pub use vector_opcode::VectorOpcode;

/// Error type used when an encoded `u32` value is not a valid prefixed opcode.
#[derive(Clone, Debug)]
pub struct InvalidPrefixedOpcode<const P: u8> {
    opcode: u32,
}

impl<const P: u8> InvalidPrefixedOpcode<P> {
    const fn new(opcode: u32) -> Self {
        Self { opcode }
    }
}

impl<const P: u8> core::fmt::Display for InvalidPrefixedOpcode<P> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#02X} {} is not a recognized opcode", P, self.opcode)
    }
}

#[cfg(feature = "std")]
impl<const P: u8> std::error::Error for InvalidPrefixedOpcode<P> {}

impl<const P: u8> From<InvalidPrefixedOpcode<P>> for crate::parser::Error {
    #[inline]
    fn from(error: InvalidPrefixedOpcode<P>) -> Self {
        crate::parser_bad_format!("{error}")
    }
}
