use crate::bytes::{Bytes, Window};
use crate::index::MemIdx;
use crate::instruction_set::InstructionSequence;
use crate::parser::{self, Offset, Result, ResultExt};
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
    count: u32,
    offset: u64,
    bytes: B,
}

impl<B: Bytes> DatasComponent<B> {
    /// Uses the given [`Bytes`] to read the contents of the *data section* of a module, starting
    /// at the specified `offset`.
    pub fn new(mut offset: u64, bytes: B) -> Result<Self> {
        Ok(Self {
            count: parser::leb128::u32(&mut offset, &bytes).context("data section count")?,
            bytes,
            offset,
        })
    }

    #[inline]
    fn parse_inner<Y, Z, M, D>(&mut self, mode_f: M, data_f: D) -> Result<Z>
    where
        M: FnOnce(&mut DataMode<&mut u64, &B>) -> Result<Y>,
        D: FnOnce(Y, Window<&B>) -> Result<Z>,
    {
        let mode_tag =
            parser::leb128::u32(&mut self.offset, &self.bytes).context("data segment mode")?;

        let mut mode = match mode_tag {
            0 => DataMode::Active(
                MemIdx::from(0u8),
                InstructionSequence::new(&mut self.offset, &self.bytes),
            ),
            _ => {
                return Err(crate::parser_bad_format!(
                    "{mode_tag} is not a supported data segment mode"
                ))
            }
        };

        let data_arg = mode_f(&mut mode)?;
        mode.finish()?;

        let data_length =
            parser::leb128::u64(&mut self.offset, &self.bytes).context("data segment length")?;

        let data = Window::new(&self.bytes, self.offset, data_length);
        let result = data_f(data_arg, data)?;

        self.offset = self.offset.checked_add(data_length).ok_or_else(|| {
            crate::parser_bad_format!(
                "expected data segment to have a length of {data_length} bytes, but end of section was unexpectedly reached"
            )
        })?;

        Ok(result)
    }

    /// Parses the next data segment in the section.
    pub fn parse<Y, Z, M, D>(&mut self, mode_f: M, data_f: D) -> Result<Option<Z>>
    where
        M: FnOnce(&mut DataMode<&mut u64, &B>) -> Result<Y>,
        D: FnOnce(Y, Window<&B>) -> Result<Z>,
    {
        if self.count == 0 {
            return Ok(None);
        }

        let result = self.parse_inner(mode_f, data_f);

        if result.is_ok() {
            self.count -= 1;
        } else {
            self.count = 0;
        }

        result.map(Some)
    }

    /// Gets the expected remaining number of entires in the *data section* that have yet to be parsed.
    #[inline]
    pub fn len(&self) -> u32 {
        self.count
    }

    /// Returns a value indicating if the *data section* is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<B: Bytes> Debug for DatasComponent<B> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut datas = DatasComponent {
            count: self.count,
            offset: self.offset,
            bytes: &self.bytes,
        };

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
                            DataMode::Active(*memory, offset.map_bytes(|_| &self.bytes))
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
