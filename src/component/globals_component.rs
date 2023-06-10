use crate::{
    bytes::Bytes,
    instruction_set::InstructionSequence,
    parser::{Result, ResultExt as _, Vector},
    types::GlobalType,
};

/// Represents the
/// [**globals** component](https://webassembly.github.io/spec/core/syntax/modules.html#globals) of
/// a WebAssembly module, stored in and parsed from the
/// [*global section*](https://webassembly.github.io/spec/core/binary/modules.html#global-section).
#[derive(Clone, Copy)]
pub struct GlobalsComponent<B: Bytes> {
    globals: Vector<u64, B>,
}

impl<B: Bytes> From<Vector<u64, B>> for GlobalsComponent<B> {
    #[inline]
    fn from(globals: Vector<u64, B>) -> Self {
        Self { globals }
    }
}

impl<B: Bytes> GlobalsComponent<B> {
    /// Uses the given [`Bytes`] to read the contents of the *global section* of a module, starting
    /// at the specified `offset`.
    pub fn new(offset: u64, bytes: B) -> Result<Self> {
        Vector::parse(offset, bytes)
            .context("at start of global section")
            .map(Self::from)
    }

    /// Parses a
    /// [WebAssembly `global`](https://webassembly.github.io/spec/core/binary/modules.html#global-section).
    pub fn parse<T, F>(&mut self, f: F) -> Result<Option<T>>
    where
        F: FnOnce(GlobalType, &mut InstructionSequence<&mut u64, &B>) -> Result<T>,
    {
        self.globals
            .advance(|offset, bytes| {
                let global_type = crate::component::global_type(offset, bytes)?;
                let mut expression = InstructionSequence::new(offset, bytes);
                let result = f(global_type, &mut expression).context("global expression")?;
                expression.finish().context("global expression")?;
                Result::Ok(result)
            })
            .transpose()
            .context("within global section")
    }

    /// Gets the expected remaining number of entires in the *global section* that have yet to be parsed.
    #[inline]
    pub fn remaining_count(&self) -> u32 {
        self.globals.remaining_count()
    }

    pub(crate) fn borrowed(&self) -> GlobalsComponent<&B> {
        self.globals.borrowed().into()
    }
}

impl<B: Bytes> core::fmt::Debug for GlobalsComponent<B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut globals = self.borrowed();

        struct Global<'a, B: Bytes> {
            r#type: GlobalType,
            init: InstructionSequence<u64, &'a B>,
        }

        impl<B: Bytes> core::fmt::Debug for Global<'_, B> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct("Global")
                    .field("type", &self.r#type)
                    .field("init", &self.init)
                    .finish()
            }
        }

        let mut list = f.debug_list();
        loop {
            let result = globals.parse(|ty, init| Ok((ty, init.cloned())));

            list.entry(&match result {
                Ok(None) => break,
                Ok(Some((ty, init))) => Ok(Global { r#type: ty, init }),
                Err(e) => Err(e),
            });
        }
        list.finish()
    }
}
