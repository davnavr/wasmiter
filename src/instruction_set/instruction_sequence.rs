use crate::allocator::{Allocator, Vector};
use crate::instruction_set::{Instruction, Opcode};
use crate::parser::{input::Input, Decoder, Result, ResultExt};

impl<I: Input> Decoder<I> {
    /// Parses a WebAssembly [`Instruction`].
    pub fn instruction<A: Allocator>(&mut self, allocator: &A) -> Result<Instruction<A>> {
        let opcode = Opcode::try_from(self.one_byte_exact().context("opcode byte")?)?;
        Ok(match opcode {
            Opcode::Unreachable => Instruction::Unreachable,
            Opcode::Nop => Instruction::Nop,
            Opcode::Block => Instruction::Block(self.block_type()?),
            Opcode::Loop => Instruction::Loop(self.block_type().context("loop block type")?),
            Opcode::If => Instruction::If(self.block_type().context("if block type")?),
            Opcode::Br => Instruction::Br(self.index().context("branch label")?),
            Opcode::BrIf => Instruction::BrIf(self.index().context("branch label")?),
            Opcode::Return => Instruction::Return,
            Opcode::Call => Instruction::Call(self.index().context("call target")?),
            Opcode::CallIndirect => Instruction::CallIndirect(
                self.index().context("indirect call signature")?,
                self.index().context("indirect call target")?,
            ),
            Opcode::Else => Instruction::Else,
            Opcode::End => Instruction::End,
            _ => todo!("{opcode:?} not implemented"),
        })
    }
}

/// Represents an expression or
/// [`expr`](https://webassembly.github.io/spec/core/syntax/instructions.html), which is a sequence
/// of instructions that is terminated by an [**end**](Instruction::End) instruction.
pub struct InstructionSequence<I: Input, A: Allocator> {
    blocks: u32,
    parser: Decoder<I>,
    current: Option<Instruction<A>>,
    allocator: A,
}

impl<I: Input, A: Allocator> InstructionSequence<I, A> {
    /// Uses the given [`Decoder`] to read a sequence of instructions, with the [`Allocator`].
    pub fn with_allocator(parser: Decoder<I>, allocator: A) -> Self {
        Self {
            blocks: 1,
            parser,
            current: None,
            allocator,
        }
    }

    /// Fetches the next [`Instruction`] in the sequence.
    pub fn next(&mut self) -> Result<Option<&Instruction<A>>> {
        if self.blocks == 0 {
            return Ok(None);
        }

        let instruction = self
            .current
            .insert(self.parser.instruction(&self.allocator)?);

        match instruction {
            Instruction::Block(_) | Instruction::Loop(_) | Instruction::If(_) => {
                self.blocks = self
                    .blocks
                    .checked_add(1)
                    .ok_or_else(|| crate::parser_bad_format!("block nesting counter overflowed"))?;
            }
            Instruction::End => {
                // Won't underflow, check for self.blocks == 0 ensures None is returned early
                self.blocks -= 1;
            }
            _ => {}
        }

        Ok(Some(instruction))
    }

    /// Processes the remaining instructions in the sequence.
    ///
    /// If the expression is not terminated by an [**end**](Instruction::End) instruction, then
    /// an error is returned.
    pub fn finish(mut self) -> Result<()> {
        while let Some(_) = self.next()? {}

        match self.blocks {
            0 => Ok(()),
            1 => Err(crate::parser_bad_format!(
                "missing end instruction for expression, or blocks were not structured correctly"
            )),
            missing => Err(crate::parser_bad_format!(
                "missing {missing} end instructions, blocks are not structured correctly"
            )),
        }
    }

    fn try_clone(&self) -> Result<InstructionSequence<I::Fork, &A>> {
        Ok(InstructionSequence {
            blocks: self.blocks,
            parser: self.parser.fork()?,
            current: todo!(), //self.current.clone(),
            allocator: &self.allocator,
        })
    }
}

impl<I: Input, A: Allocator> Iterator for InstructionSequence<I, A> {
    type Item = Result<Instruction<A>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next() {
            Err(e) => Some(Err(e)),
            Ok(_) => self.current.take().map(Ok),
        }
    }
}

impl<I: Input, A: Allocator> core::fmt::Debug for InstructionSequence<I, A> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("InstructionSequence")
            .finish_non_exhaustive()
    }
}
