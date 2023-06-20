use crate::{
    input::{BorrowInput, CloneInput, HasInput, Input},
    instruction_set::InstructionSequence,
    parser::{MixedResult, Parsed, ResultExt as _, Vector},
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
    pub fn new(offset: u64, input: I) -> Parsed<Self> {
        Vector::parse(offset, input)
            .context("at start of global section")
            .map(Self::from)
    }

    /// Parses a WebAssembly [`global`] with the given closure, allowing for custom errors.
    ///
    /// See the documentation for [`GlobalsComponent::parse`] for more information.
    ///
    /// [`global`]: https://webassembly.github.io/spec/core/binary/modules.html#global-section
    pub fn parse_mixed<E, T, F>(&mut self, f: F) -> MixedResult<Option<T>, E>
    where
        F: FnOnce(GlobalType, &mut InstructionSequence<&mut u64, &I>) -> MixedResult<T, E>,
    {
        #[inline]
        fn parse_inner<I, E, T, F>(offset: &mut u64, input: &I, f: F) -> MixedResult<T, E>
        where
            I: Input,
            F: FnOnce(GlobalType, &mut InstructionSequence<&mut u64, &I>) -> MixedResult<T, E>,
        {
            let global_type = crate::component::global_type(offset, input)?;
            let mut expression = InstructionSequence::new(offset, input);
            let result = f(global_type, &mut expression)?;
            expression.finish()?;
            Ok(result)
        }

        self.globals
            .advance(|offset, input| {
                let start = *offset;
                parse_inner(offset, input, f).with_location_context("global section", start)
            })
            .transpose()
    }

    /// Parses a WebAssembly [`global`] with the given closure.
    ///
    /// [`global`]: https://webassembly.github.io/spec/core/binary/modules.html#global-section
    #[inline]
    pub fn parse<T, F>(&mut self, f: F) -> Parsed<Option<T>>
    where
        F: FnOnce(GlobalType, &mut InstructionSequence<&mut u64, &I>) -> Parsed<T>,
    {
        self.parse_mixed(|gt, init| f(gt, init).map_err(Into::into))
            .map_err(Into::into)
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

impl<'a, I: Clone + Input + 'a> CloneInput<'a, I> for GlobalsComponent<&'a I> {
    type Cloned = GlobalsComponent<I>;

    #[inline]
    fn clone_input(&self) -> Self::Cloned {
        self.globals.clone_input().into()
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
