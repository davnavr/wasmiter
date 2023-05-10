use crate::component::GlobalType;
use crate::instruction_set::InstructionSequence;
use crate::parser::{input::Input, Decoder, Result, ResultExt};

/// Represents the
/// [**globals** component](https://webassembly.github.io/spec/core/syntax/modules.html#globals) of
/// a WebAssembly module, stored in and parsed from the
/// [*global section*](https://webassembly.github.io/spec/core/binary/modules.html#global-section).
pub struct GlobalsComponent<I: Input> {
    count: usize,
    decoder: Decoder<I>,
}

impl<I: Input> GlobalsComponent<I> {
    /// Uses the given [`Decoder<I>`] to read the contents of the *global section* of a module.
    pub fn new(mut decoder: Decoder<I>) -> Result<Self> {
        Ok(Self {
            count: decoder.leb128_usize().context("global section count")?,
            decoder,
        })
    }

    fn next_inner<T, F>(&mut self, f: F) -> Result<T>
    where
        F: FnOnce(GlobalType, &mut InstructionSequence<&mut I>) -> Result<T>,
    {
        let global_type = self.decoder.global_type()?;
        let mut expression = InstructionSequence::new(self.decoder.by_ref());
        let result = f(global_type, &mut expression).context("global expression")?;
        expression.finish().context("global expression")?;
        Ok(result)
    }

    /// Parses a
    /// [WebAssembly `global`](https://webassembly.github.io/spec/core/binary/modules.html#global-section).
    pub fn next<T, F>(&mut self, f: F) -> Result<Option<T>>
    where
        F: FnOnce(GlobalType, &mut InstructionSequence<&mut I>) -> Result<T>,
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

    // fn try_clone(&self) -> Result<GlobalsComponent<I::Fork>> {
    //     Ok(GlobalsComponent {
    //         count: self.count,
    //         decoder: self.decoder.fork()?,
    //     })
    // }
}

impl<I: Input> core::fmt::Debug for GlobalsComponent<I> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("GlobalsComponent").finish_non_exhaustive()
    }
}
