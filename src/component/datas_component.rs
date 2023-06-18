use crate::{
    index::MemIdx,
    input::{BorrowInput, CloneInput, HasInput, Input, Window},
    instruction_set::InstructionSequence,
    parser::{self, Offset, Result, ResultExt as _, Vector},
};
use core::fmt::{Debug, Formatter};

/// Specifies the mode of a
/// [data segment](https://webassembly.github.io/spec/core/syntax/modules.html#data-segments).
#[derive(Clone, Copy)]
pub enum DataMode<O: Offset, I: Input> {
    /// A **passive** data segment's elements are copied to a memory using the
    /// [`memory.init`](crate::instruction_set::Instruction::MemoryInit) instruction.
    Passive,
    /// An **active** data segment copies elements into the specified memory, starting at the
    /// offset specified by an expression, during
    /// [instantiation](https://webassembly.github.io/spec/core/exec/modules.html#exec-instantiation)
    /// of the module.
    Active(MemIdx, InstructionSequence<O, I>),
}

impl<O: Offset, I: Input> DataMode<O, I> {
    fn finish(self) -> Result<()> {
        match self {
            Self::Passive => (),
            Self::Active(_, offset) => {
                offset.finish()?;
            }
        }
        Ok(())
    }
}

impl<O: Offset, I: Input> Debug for DataMode<O, I> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Passive => f.debug_tuple("Passive").finish(),
            Self::Active(memory, offset) => f
                .debug_struct("Active")
                .field("memory", memory)
                .field("offset", offset)
                .finish(),
        }
    }
}

/// Represents the
/// [**datas** component](https://webassembly.github.io/spec/core/syntax/modules.html#data-segments)
/// of a WebAssembly module, stored in and parsed from the
/// [*data section*](https://webassembly.github.io/spec/core/binary/modules.html#data-section).
#[derive(Clone, Copy)]
pub struct DatasComponent<I: Input> {
    entries: Vector<u64, I>,
}

impl<I: Input> From<Vector<u64, I>> for DatasComponent<I> {
    #[inline]
    fn from(entries: Vector<u64, I>) -> Self {
        Self { entries }
    }
}

impl<I: Input> DatasComponent<I> {
    /// Uses the given [`Input`] to read the contents of the *data section* of a module, starting
    /// at the specified `offset`.
    pub fn new(offset: u64, input: I) -> Result<Self> {
        Vector::parse(offset, input)
            .context("at start of data section")
            .map(Self::from)
    }

    /// Parses the next data segment in the section.
    pub fn parse<Y, Z, M, D>(&mut self, mode_f: M, data_f: D) -> Result<Option<Z>>
    where
        M: FnOnce(&mut DataMode<&mut u64, &I>) -> Result<Y>,
        D: FnOnce(Y, Window<&I>) -> Result<Z>,
    {
        self.entries.advance(|offset, input| {
            let mode_offset = *offset;
            let mode_tag=
                parser::leb128::u32(offset, input).context("while parsing data segment mode")?;

            let mut copied_offset = *offset;
            let mut mode: DataMode<&mut u64, &I> = match mode_tag {
                0 => DataMode::Active(
                    MemIdx::from(0u8),
                    InstructionSequence::new(&mut copied_offset, input),
                ),
                1 => DataMode::Passive,
                2 => DataMode::Active(
                    crate::component::index(&mut copied_offset, input).context("could not parse target memory of active data segment")?,
                    InstructionSequence::new(&mut copied_offset, input),
                ),
                _ => {
                    #[inline(never)]
                    #[cold]
                    fn unsupported_mode(offset: u64, mode: u32) -> parser::Error {
                        parser::Error::new(parser::ErrorKind::BadDataSegmentMode(mode)).with_location_context("data segment", offset)
                    }

                    return Err(unsupported_mode(mode_offset, mode_tag));
                }
            };

            let data_arg = mode_f(&mut mode)?;
            mode.finish()?;
            *offset = copied_offset;

            let data_length =
                parser::leb128::u64(offset, input).context("data segment length")?;

            let data = Window::with_offset_and_length(input, *offset, data_length);
            let result = data_f(data_arg, data)?;

            crate::input::increment_offset(offset, data_length).with_context(|| move |f| {
                write!(f, "expected data segment to have a length of {data_length} bytes, but end of section was unexpectedly reached")
            })?;

            Ok(result)
        }).transpose().context("within data section")
    }

    /// Gets the expected remaining number of entires in the *data section* that have yet to be parsed.
    #[inline]
    pub fn remaining_count(&self) -> u32 {
        self.entries.remaining_count()
    }
}

impl<I: Input> HasInput<I> for DatasComponent<I> {
    #[inline]
    fn input(&self) -> &I {
        self.entries.input()
    }
}

impl<'a, I: Input + 'a> BorrowInput<'a, I> for DatasComponent<I> {
    type Borrowed = DatasComponent<&'a I>;

    #[inline]
    fn borrow_input(&'a self) -> Self::Borrowed {
        self.entries.borrow_input().into()
    }
}

impl<I: Input> Debug for DatasComponent<I> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut datas = self.borrow_input();

        struct DataSegment<'a, I: Input> {
            mode: DataMode<u64, I>,
            data: Window<&'a I>,
        }

        impl<I: Input> Debug for DataSegment<'_, I> {
            fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
                f.debug_struct("DataSegment")
                    .field("mode", &self.mode)
                    .field("data", &self.data)
                    .finish()
            }
        }

        let mut list = f.debug_list();
        loop {
            let result = datas.parse(
                |mode| {
                    Ok(match mode {
                        DataMode::Passive => DataMode::Passive,
                        DataMode::Active(memory, offset) => {
                            DataMode::Active(*memory, offset.clone_input())
                        }
                    })
                },
                |mode, data| {
                    list.entry(&Result::Ok(DataSegment { mode, data }));
                    Ok(())
                },
            );

            match result {
                Ok(Some(())) => (),
                Ok(None) => break,
                Err(e) => {
                    list.entry(&Result::<()>::Err(e));
                    break;
                }
            }
        }
        list.finish()
    }
}
