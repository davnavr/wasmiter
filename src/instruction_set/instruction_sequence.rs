use crate::instruction_set::{Instruction, Opcode};
use crate::parser::{input::Input, Parser, Result, ResultExt};

impl<I: Input> Parser<I> {
    /// Parses a WebAssembly [`Instruction`].
    pub fn instruction(&mut self) -> Result<Instruction> {
        let opcode = Opcode::try_from(self.one_byte_exact().context("opcode byte")?)?;
        Ok(match opcode {
            Opcode::Unreachable => Instruction::Unreachable,
            Opcode::Nop => Instruction::Nop,
            _ => todo!("{opcode:?} not implemented"),
        })
    }
}

/// Represents an expression or
/// [`expr`](https://webassembly.github.io/spec/core/syntax/instructions.html), which is a sequence
/// of instructions that is terminated by an [**end**](Instruction::End) instruction.
pub struct InstructionSequence<I: Input> {
    blocks: u32,
    parser: Parser<I>,
    current: Option<Instruction>,
}

impl<I: Input> InstructionSequence<I> {
    /// Uses the given [`Parser`] to read a sequence of instructions.
    pub fn new(parser: Parser<I>) -> Self {
        Self {
            blocks: 1,
            parser,
            current: None,
        }
    }

    /// Fetches the next [`Instruction`] in the sequence.
    pub fn next(&mut self) -> Result<Option<&Instruction>> {
        if self.blocks == 0 {
            return Ok(None);
        }

        let instruction = self.current.insert(self.parser.instruction()?);

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

    fn try_clone(&self) -> Result<InstructionSequence<I::Fork>> {
        Ok(InstructionSequence {
            blocks: self.blocks,
            parser: self.parser.fork()?,
            current: self.current.clone(),
        })
    }
}

impl<I: Input> Iterator for InstructionSequence<I> {
    type Item = Result<Instruction>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next() {
            Err(e) => Some(Err(e)),
            Ok(_) => self.current.take().map(Ok),
        }
    }
}

impl<I: Input> core::fmt::Debug for InstructionSequence<I> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        crate::component::debug_section_contents(self.try_clone(), f)
    }
}
