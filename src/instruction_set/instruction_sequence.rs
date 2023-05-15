use crate::bytes::Bytes;
use crate::component;
use crate::instruction_set::{self, FCPrefixedOpcode, Instruction, Opcode};
use crate::parser::{self, leb128, Result, ResultExt, Vector};

fn memarg<B: Bytes>(offset: &mut u64, bytes: &B) -> Result<instruction_set::MemArg> {
    let a = leb128::u32(offset, bytes).context("memory argument alignment")?;
    let o = leb128::u32(offset, bytes).context("memory argument offset")?;

    let align = u8::try_from(a)
        .ok()
        .and_then(core::num::NonZeroU8::new)
        .ok_or_else(|| {
            crate::parser_bad_format!("{a} is too large to be a valid alignment power")
        })?;

    Ok(instruction_set::MemArg::new(o, align))
}

/// Parses a WebAssembly [`Instruction`].
///
/// In order to ensure the instruction is completely parsed, callers should call
/// [Instruction::finish].
fn instruction<'a, 'b, B: Bytes>(
    offset: &'a mut u64,
    bytes: &'b B,
) -> Result<Instruction<'a, &'b B>> {
    let opcode = Opcode::try_from(parser::one_byte_exact(offset, bytes).context("opcode byte")?)?;
    Ok(match opcode {
        Opcode::Unreachable => Instruction::Unreachable,
        Opcode::Nop => Instruction::Nop,
        Opcode::Block => Instruction::Block(component::block_type(offset, bytes)?),
        Opcode::Loop => {
            Instruction::Loop(component::block_type(offset, bytes).context("loop block type")?)
        }
        Opcode::If => {
            Instruction::If(component::block_type(offset, bytes).context("if block type")?)
        }
        Opcode::Br => Instruction::Br(component::index(offset, bytes)?),
        Opcode::BrIf => Instruction::BrIf(component::index(offset, bytes)?),
        Opcode::BrTable => Instruction::BrTable(
            Vector::new(offset, bytes, Default::default()).context("branch table")?,
        ),
        Opcode::Return => Instruction::Return,
        Opcode::Call => Instruction::Call(component::index(offset, bytes).context("call target")?),
        Opcode::CallIndirect => Instruction::CallIndirect(
            component::index(offset, bytes).context("indirect call signature")?,
            component::index(offset, bytes).context("indirect call target")?,
        ),
        Opcode::Else => Instruction::Else,
        Opcode::End => Instruction::End,
        Opcode::Drop => Instruction::Drop,
        Opcode::Select => {
            Instruction::Select(Vector::empty_with_offset(offset, bytes, Default::default()))
        }
        Opcode::SelectMany => Instruction::Select(
            Vector::new(offset, bytes, Default::default()).context("select types")?,
        ),
        Opcode::LocalGet => Instruction::LocalGet(component::index(offset, bytes)?),
        Opcode::LocalSet => Instruction::LocalSet(component::index(offset, bytes)?),
        Opcode::LocalTee => Instruction::LocalTee(component::index(offset, bytes)?),
        Opcode::GlobalGet => Instruction::GlobalGet(component::index(offset, bytes)?),
        Opcode::GlobalSet => Instruction::GlobalSet(component::index(offset, bytes)?),
        Opcode::TableGet => Instruction::TableGet(component::index(offset, bytes)?),
        Opcode::TableSet => Instruction::TableSet(component::index(offset, bytes)?),
        Opcode::I32Load => Instruction::I32Load(memarg(offset, bytes)?),
        Opcode::I64Load => Instruction::I64Load(memarg(offset, bytes)?),
        Opcode::F32Load => Instruction::F32Load(memarg(offset, bytes)?),
        Opcode::F64Load => Instruction::F64Load(memarg(offset, bytes)?),
        Opcode::I32Load8S => Instruction::I32Load8S(memarg(offset, bytes)?),
        Opcode::I32Load8U => Instruction::I32Load8U(memarg(offset, bytes)?),
        Opcode::I32Load16S => Instruction::I32Load16S(memarg(offset, bytes)?),
        Opcode::I32Load16U => Instruction::I32Load16U(memarg(offset, bytes)?),
        Opcode::I64Load8S => Instruction::I64Load8S(memarg(offset, bytes)?),
        Opcode::I64Load8U => Instruction::I64Load8U(memarg(offset, bytes)?),
        Opcode::I64Load16S => Instruction::I64Load16S(memarg(offset, bytes)?),
        Opcode::I64Load16U => Instruction::I64Load16U(memarg(offset, bytes)?),
        Opcode::I64Load32S => Instruction::I64Load32S(memarg(offset, bytes)?),
        Opcode::I64Load32U => Instruction::I64Load32U(memarg(offset, bytes)?),
        Opcode::I32Store => Instruction::I32Store(memarg(offset, bytes)?),
        Opcode::I64Store => Instruction::I64Store(memarg(offset, bytes)?),
        Opcode::F32Store => Instruction::F32Store(memarg(offset, bytes)?),
        Opcode::F64Store => Instruction::F64Store(memarg(offset, bytes)?),
        Opcode::I32Store8 => Instruction::I32Store8(memarg(offset, bytes)?),
        Opcode::I32Store16 => Instruction::I32Store16(memarg(offset, bytes)?),
        Opcode::I64Store8 => Instruction::I64Store8(memarg(offset, bytes)?),
        Opcode::I64Store16 => Instruction::I64Store16(memarg(offset, bytes)?),
        Opcode::I64Store32 => Instruction::I64Store32(memarg(offset, bytes)?),
        Opcode::MemorySize => Instruction::MemorySize(component::index(offset, bytes)?),
        Opcode::MemoryGrow => Instruction::MemoryGrow(component::index(offset, bytes)?),
        Opcode::I32Const => Instruction::I32Const(leb128::s32(offset, bytes)?),
        Opcode::I64Const => Instruction::I64Const(leb128::s64(offset, bytes)?),
        Opcode::F32Const => Instruction::F32Const(f32::from_le_bytes(
            parser::byte_array(offset, bytes).context("32-bit float constant")?,
        )),
        Opcode::F64Const => Instruction::F64Const(f64::from_le_bytes(
            parser::byte_array(offset, bytes).context("64-bit float constant")?,
        )),
        Opcode::RefNull => {
            Instruction::RefNull(component::ref_type(offset, bytes).context("type for null")?)
        }
        Opcode::RefIsNull => Instruction::RefIsNull,
        Opcode::RefFunc => Instruction::RefFunc(
            component::index(offset, bytes).context("invalid reference to function")?,
        ),
        Opcode::PrefixFC => {
            let actual_opcode = leb128::u32(offset, bytes)
                .context("actual opcode")?
                .try_into()?;

            match actual_opcode {
                FCPrefixedOpcode::MemoryInit => Instruction::MemoryInit(
                    component::index(offset, bytes)?,
                    component::index(offset, bytes)?,
                ),
                FCPrefixedOpcode::DataDrop => {
                    Instruction::DataDrop(component::index(offset, bytes)?)
                }
                FCPrefixedOpcode::MemoryCopy => Instruction::MemoryCopy {
                    destination: component::index(offset, bytes).context("destination memory")?,
                    source: component::index(offset, bytes).context("source memory")?,
                },
                FCPrefixedOpcode::MemoryFill => {
                    Instruction::MemoryFill(component::index(offset, bytes)?)
                }
                FCPrefixedOpcode::TableInit => Instruction::TableInit(
                    component::index(offset, bytes)?,
                    component::index(offset, bytes)?,
                ),
                FCPrefixedOpcode::ElemDrop => {
                    Instruction::ElemDrop(component::index(offset, bytes)?)
                }
                FCPrefixedOpcode::TableCopy => Instruction::TableCopy {
                    destination: component::index(offset, bytes).context("destination table")?,
                    source: component::index(offset, bytes).context("source table")?,
                },
                FCPrefixedOpcode::TableGrow => {
                    Instruction::TableGrow(component::index(offset, bytes)?)
                }
                FCPrefixedOpcode::TableSize => {
                    Instruction::TableSize(component::index(offset, bytes)?)
                }
                FCPrefixedOpcode::TableFill => {
                    Instruction::TableFill(component::index(offset, bytes)?)
                }
                _ => todo!("0xFC {actual_opcode:?} not implemented"),
            }
        }
        _ => todo!("{opcode:?} not implemented"),
    }) //.context() // the opcode name
}

/// Represents an expression or
/// [`expr`](https://webassembly.github.io/spec/core/syntax/instructions.html), which is a sequence
/// of instructions that is terminated by an [**end**](Instruction::End) instruction.
pub struct InstructionSequence<B: Bytes> {
    blocks: u32,
    offset: u64,
    bytes: B,
}

impl<B: Bytes> InstructionSequence<B> {
    /// Uses the given [`Bytes`] to read a sequence of instructions, starting at the given
    /// `offset`.
    pub fn new(offset: u64, bytes: B) -> Self {
        Self {
            blocks: 1,
            offset,
            bytes,
        }
    }

    /// Returns a value indicating if there are more instructions remaining to be parsed
    #[inline]
    pub fn is_finished(&self) -> bool {
        self.blocks == 0
    }

    #[inline]
    fn process_next<'a, F>(&'a mut self, f: F) -> Result<()>
    where
        F: FnOnce(&mut Instruction<'a, &'a B>) -> Result<()>,
    {
        let mut instruction = self::instruction(&mut self.offset, &self.bytes)?;
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
    pub fn next<'a, F>(&'a mut self, f: F) -> Option<Result<()>>
    where
        F: FnOnce(&mut Instruction<'a, &'a B>) -> Result<()>,
    {
        if self.is_finished() {
            return None;
        }

        Some(self.process_next(f))
    }

    /// Processes the remaining instructions in the sequence. Returns the offset to the byte after
    /// the last byte of the last instruction.
    ///
    /// If the expression is not terminated by an [**end**](Instruction::End) instruction, then
    /// an error is returned.
    pub fn finish(mut self) -> Result<u64> {
        loop {
            match self.next(|_| Ok(())) {
                Some(Ok(())) => (),
                Some(Err(e)) => return Err(e),
                None => break,
            }
        }

        match self.blocks {
            0 => Ok(self.offset),
            1 => Err(crate::parser_bad_format!(
                "missing end instruction for expression, or blocks were not structured correctly"
            )),
            missing => Err(crate::parser_bad_format!(
                "missing {missing} end instructions, blocks are not structured correctly"
            )),
        }
    }

    // fn try_clone(&self) -> Result<InstructionSequence<I::Fork>> {
    //     Ok(InstructionSequence {
    //         blocks: self.blocks,
    //         parser: self.parser.fork()?,
    //     })
    // }
}

impl<B: Bytes> core::fmt::Debug for InstructionSequence<B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("InstructionSequence")
            .finish_non_exhaustive()
    }
}
