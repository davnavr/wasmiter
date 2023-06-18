use crate::{
    input::{BorrowInput, CloneInput, HasInput, Input},
    instruction_set::InstructionSequence,
    parser::{Result, ResultExt as _, Vector},
    types::GlobalType,
};

/// Represents the
/// [**globals** component](https://webassembly.github.io/spec/core/syntax/modules.html#globals) of
/// a WebAssembly module, stored in and parsed from the
/// [*global section*](https://webassembly.github.io/spec/core/binary/modules.html#global-section).
#[derive(Clone, Copy)]
pub struct GlobalsComponent<I: Input> {
    globals: Vector<u64, I>,
}

impl<I: Input> From<Vector<u64, I>> for GlobalsComponent<I> {
    #[inline]
    fn from(globals: Vector<u64, I>) -> Self {
        Self { globals }
    }
}

impl<I: Input> GlobalsComponent<I> {
    /// Uses the given [`Input`] to read the contents of the *global section* of a module, starting
    /// at the specified `offset`.
    pub fn new(offset: u64, input: I) -> Result<Self> {
        Vector::parse(offset, input)
            .context("at start of global section")
            .map(Self::from)
    }

    /// Parses a
    /// [WebAssembly `global`](https://webassembly.github.io/spec/core/binary/modules.html#global-section).
    pub fn parse<T, F>(&mut self, f: F) -> Result<Option<T>>
    where
        F: FnOnce(GlobalType, &mut InstructionSequence<&mut u64, &I>) -> Result<T>,
    {
        self.globals
            .advance(|offset, input| {
                let global_type = crate::component::global_type(offset, input)?;
                let mut expression = InstructionSequence::new(offset, input);
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
}

impl<I: Input> HasInput<I> for GlobalsComponent<I> {
    #[inline]
    fn input(&self) -> &I {
        self.globals.input()
    }
}

impl<'a, I: Input + 'a> BorrowInput<'a, I> for GlobalsComponent<I> {
    type Borrowed = GlobalsComponent<&'a I>;

    #[inline]
    fn borrow_input(&'a self) -> Self::Borrowed {
        self.globals.borrow_input().into()
    }
}

impl<I: Input> core::fmt::Debug for GlobalsComponent<I> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut globals = self.borrow_input();

        struct Global<'a, I: Input> {
            r#type: GlobalType,
            init: InstructionSequence<u64, &'a I>,
        }

        impl<I: Input> core::fmt::Debug for Global<'_, I> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct("Global")
                    .field("type", &self.r#type)
                    .field("init", &self.init)
                    .finish()
            }
        }

        let mut list = f.debug_list();
        loop {
            let result = globals.parse(|ty, init| Ok((ty, init.clone_input())));

            list.entry(&match result {
                Ok(None) => break,
                Ok(Some((ty, init))) => Ok(Global { r#type: ty, init }),
                Err(e) => Err(e),
            });
        }
        list.finish()
    }
}
