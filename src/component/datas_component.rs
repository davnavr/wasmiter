use crate::{
    bytes::{Bytes, Window},
    index::MemIdx,
    instruction_set::InstructionSequence,
    parser::{self, Offset, Result, ResultExt as _, Vector},
};
use core::fmt::{Debug, Formatter};

/// Specifies the mode of a
/// [data segment](https://webassembly.github.io/spec/core/syntax/modules.html#data-segments).
#[derive(Clone, Copy)]
pub enum DataMode<O: Offset, B: Bytes> {
    /// A **passive** data segment's elements are copied to a memory using the
    /// [`memory.init`](crate::instruction_set::Instruction::MemoryInit) instruction.
    Passive,
    /// An **active** data segment copies elements into the specified memory, starting at the
    /// offset specified by an expression, during
    /// [instantiation](https://webassembly.github.io/spec/core/exec/modules.html#exec-instantiation)
    /// of the module.
    Active(MemIdx, InstructionSequence<O, B>),
}

impl<O: Offset, B: Bytes> DataMode<O, B> {
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

impl<O: Offset, B: Bytes> Debug for DataMode<O, B> {
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
pub struct DatasComponent<B: Bytes> {
    entries: Vector<u64, B>,
}

impl<B: Bytes> From<Vector<u64, B>> for DatasComponent<B> {
    #[inline]
    fn from(entries: Vector<u64, B>) -> Self {
        Self { entries }
    }
}

impl<B: Bytes> DatasComponent<B> {
    /// Uses the given [`Bytes`] to read the contents of the *data section* of a module, starting
    /// at the specified `offset`.
    pub fn new(offset: u64, bytes: B) -> Result<Self> {
        Vector::parse(offset, bytes)
            .context("at start of data section")
            .map(Self::from)
    }

    /// Parses the next data segment in the section.
    pub fn parse<Y, Z, M, D>(&mut self, mode_f: M, data_f: D) -> Result<Option<Z>>
    where
        M: FnOnce(&mut DataMode<&mut u64, &B>) -> Result<Y>,
        D: FnOnce(Y, Window<&B>) -> Result<Z>,
    {
        self.entries.advance(|offset, bytes| {
            let mode_offset = *offset;
            let mode_tag=
                parser::leb128::u32(offset, bytes).context("while parsing data segment mode")?;

            let mut copied_offset = *offset;
            let mut mode: DataMode<&mut u64, &B> = match mode_tag {
                0 => DataMode::Active(
                    MemIdx::from(0u8),
                    InstructionSequence::new(&mut copied_offset, bytes),
                ),
                1 => DataMode::Passive,
                2 => DataMode::Active(
                    crate::component::index(&mut copied_offset, bytes).context("could not parse target memory of active data segment")?,
                    InstructionSequence::new(&mut copied_offset, bytes),
                ),
                _ => {
                    return Err(crate::parser_bad_format_at_offset!(
                        "file" @ mode_offset,
                        "{mode_tag} is not a supported data segment mode"
                    ))
                }
            };

            let data_arg = mode_f(&mut mode)?;
            mode.finish()?;
            *offset = copied_offset;

            let data_length =
                parser::leb128::u64(offset, bytes).context("data segment length")?;

            let data = Window::new(bytes, *offset, data_length);
            let result = data_f(data_arg, data)?;

            *offset = offset.checked_add(data_length).ok_or_else(|| {
                crate::parser_bad_format_at_offset!(
                    "file" @ offset,
                    "expected data segment to have a length of {data_length} bytes, but end of section was unexpectedly reached"
                )
            })?;

            Ok(result)
        }).transpose().context("within data section")
    }

    /// Gets the expected remaining number of entires in the *data section* that have yet to be parsed.
    #[inline]
    pub fn remaining_count(&self) -> u32 {
        self.entries.remaining_count()
    }

    #[inline]
    pub(crate) fn borrowed(&self) -> DatasComponent<&B> {
        self.entries.borrowed().into()
    }
}

impl<B: Bytes> Debug for DatasComponent<B> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut datas = self.borrowed();

        struct DataSegment<'a, B: Bytes> {
            mode: DataMode<u64, B>,
            data: Window<&'a B>,
        }

        impl<B: Bytes> Debug for DataSegment<'_, B> {
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
                            DataMode::Active(*memory, offset.cloned())
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
