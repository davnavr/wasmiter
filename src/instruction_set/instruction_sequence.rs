use crate::instruction_set::{Instruction, Opcode};
use crate::parser::{input::Input, Decoder, Result, ResultExt, Vector};

impl<I: Input> Decoder<I> {
    /// Parses a WebAssembly [`Instruction`].
    ///
    /// In order to ensure the instruction is completely parsed, callers should call
    /// [Instruction::finish].
    pub fn instruction(&mut self) -> Result<Instruction<&mut I>> {
        let opcode = Opcode::try_from(self.one_byte_exact().context("opcode byte")?)?;
        Ok(match opcode {
            Opcode::Unreachable => Instruction::Unreachable,
            Opcode::Nop => Instruction::Nop,
            Opcode::Block => Instruction::Block(self.block_type()?),
            Opcode::Loop => Instruction::Loop(self.block_type().context("loop block type")?),
            Opcode::If => Instruction::If(self.block_type().context("if block type")?),
            Opcode::Br => Instruction::Br(self.index().context("branch label")?),
            Opcode::BrIf => Instruction::BrIf(self.index().context("conditional branch label")?),
            Opcode::BrTable => Instruction::BrTable(
                Vector::new(self.by_ref(), Default::default()).context("branch table")?,
            ),
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
pub struct InstructionSequence<I: Input> {
    blocks: u32,
    parser: Decoder<I>,
}

impl<I: Input> InstructionSequence<I> {
    /// Uses the given [`Decoder`] to read a sequence of instructions.
    pub fn new(parser: Decoder<I>) -> Self {
        Self { blocks: 1, parser }
    }

    /// Returns a value indicating if there are more instructions remaining to be parsed
    #[inline]
    pub fn is_finished(&self) -> bool {
        self.blocks == 0
    }

    #[inline]
    fn process_next<'a, F>(&'a mut self, f: F) -> Result<()>
    where
        F: FnOnce(&mut Instruction<&'a mut I>) -> Result<()>,
    {
        let mut instruction = self.parser.instruction()?;
        f(&mut instruction)?;

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

        instruction.finish()
    }

    /// Processes the next [`Instruction`] in the sequence, providing it to the given closure.
    pub fn next<'a, F>(&'a mut self, f: F) -> Option<Result<()>> // TODO: Fix, caller can "replace" current instruction
    where
        F: FnOnce(&mut Instruction<&'a mut I>) -> Result<()>,
    {
        if self.is_finished() {
            return None;
        }

        Some(self.process_next(f))
    }

    /// Processes the remaining instructions in the sequence.
    ///
    /// If the expression is not terminated by an [**end**](Instruction::End) instruction, then
    /// an error is returned.
    pub fn finish(mut self) -> Result<()> {
        loop {
            match self.next(|_| Ok(())) {
                Some(Ok(())) => (),
                Some(Err(e)) => return Err(e),
                None => break,
            }
        }

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

    fn try_clone(&self) -> Result<InstructionSequence<I::Fork>> {
        Ok(InstructionSequence {
            blocks: self.blocks,
            parser: self.parser.fork()?,
        })
    }
}

impl<I: Input> core::fmt::Debug for InstructionSequence<I> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("InstructionSequence")
            .finish_non_exhaustive()
    }
}
