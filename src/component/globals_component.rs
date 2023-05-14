use crate::bytes::Bytes;
use crate::component::{self, GlobalType};
use crate::instruction_set::InstructionSequence;
use crate::parser::{self, Result, ResultExt};

/// Represents the
/// [**globals** component](https://webassembly.github.io/spec/core/syntax/modules.html#globals) of
/// a WebAssembly module, stored in and parsed from the
/// [*global section*](https://webassembly.github.io/spec/core/binary/modules.html#global-section).
#[derive(Clone, Copy)]
pub struct GlobalsComponent<B: Bytes> {
    count: u32,
    offset: u64,
    bytes: B,
}

impl<B: Bytes> GlobalsComponent<B> {
    /// Uses the given [`Bytes`] to read the contents of the *global section* of a module, starting
    /// at the specified `offset`.
    pub fn new(mut offset: u64, bytes: B) -> Result<Self> {
        Ok(Self {
            count: parser::leb128::u32(&mut offset, &bytes).context("global section count")?,
            offset,
            bytes,
        })
    }

    fn next_inner<T, F>(&mut self, f: F) -> Result<T>
    where
        F: FnOnce(GlobalType, &mut InstructionSequence<&B>) -> Result<T>,
    {
        let global_type = component::global_type(&mut self.offset, &self.bytes)?;
        let mut expression = InstructionSequence::new(self.offset, &self.bytes);
        let result = f(global_type, &mut expression).context("global expression")?;
        self.offset = expression.finish().context("global expression")?;
        Ok(result)
    }

    /// Parses a
    /// [WebAssembly `global`](https://webassembly.github.io/spec/core/binary/modules.html#global-section).
    pub fn next<T, F>(&mut self, f: F) -> Result<Option<T>>
    where
        F: FnOnce(GlobalType, &mut InstructionSequence<&B>) -> Result<T>,
    {
        if self.count == 0 {
            return Ok(None);
        }

        let result = self.next_inner(f);

        if result.is_ok() {
            self.count -= 1;
        } else {
            self.count = 0;
        }

        result.map(Some)
    }
}

impl<B: Bytes> core::fmt::Debug for GlobalsComponent<B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("GlobalsComponent").finish_non_exhaustive()
    }
}
