use crate::{
    component,
    input::{BorrowInput, CloneInput, HasInput, Input, Window},
    instruction_set::InstructionSequence,
    parser::{self, MixedResult, Parsed, ResultExt as _, Vector},
};
use core::fmt::{Debug, Formatter};

/// Represents an entry
/// in the
/// [*code section*], also known as a
/// [`func`](https://webassembly.github.io/spec/core/binary/modules.html#code-section).
///
/// To allow reading the code section in parallel and skipping of entries, a [`Code`] stores the
/// size, in bytes, of its contents.
///
/// To read [`Code`] from the [*code section*], see the [`CodeSection`] struct.
///
/// [*code section*]: https://webassembly.github.io/spec/core/binary/modules.html#code-section
#[derive(Clone, Copy)]
pub struct Code<I: Input> {
    index: u32,
    content: Window<I>,
}

impl<I: Input> Code<I> {
    /// The index of this *code section* entry.
    #[inline]
    pub fn index(&self) -> u32 {
        self.index
    }

    /// Gets the binary contents of this *code section* entry.
    #[inline]
    pub fn content(&self) -> &Window<I> {
        &self.content
    }

    #[inline]
    fn parse_inner<Y, Z, E>(
        &self,
        locals_f: impl FnOnce(&mut component::Locals<&mut u64, &Window<I>>) -> MixedResult<Y, E>,
        code_f: impl FnOnce(Y, &mut InstructionSequence<&mut u64, &Window<I>>) -> MixedResult<Z, E>,
    ) -> MixedResult<Z, E> {
        let mut offset = self.content.base();
        let mut locals = component::Locals::new(&mut offset, &self.content)?;
        let code_arg = locals_f(&mut locals)?;
        locals.finish()?;

        let mut code = InstructionSequence::new(&mut offset, &self.content);
        let result = code_f(code_arg, &mut code)?;

        let (_, final_offset) = code.finish()?;
        let final_length = *final_offset - self.content.base();
        let expected_length = self.content.length();
        if final_length != expected_length {
            #[inline(never)]
            #[cold]
            fn unused_bytes(expected_length: u64, actual_length: u64) -> parser::Error {
                parser::Error::new(parser::ErrorKind::InvalidFormat).with_context(parser::Context::from_closure(move |f| write!(f,
                    "expected code entry content to have a length of {expected_length} bytes, but got {actual_length}",
                )))
            }

            Err(unused_bytes(expected_length, final_length).into())
        } else {
            Ok(result)
        }
    }

    /// Parses the contents of this code entry using the given closures, allowing for custom
    /// errors.
    ///
    /// See the documentation for [`Code::parse`] for more information.
    pub fn parse_mixed<E, Y, Z, L, C>(&self, locals_f: L, code_f: C) -> MixedResult<Z, E>
    where
        L: FnOnce(&mut component::Locals<&mut u64, &Window<I>>) -> MixedResult<Y, E>,
        C: FnOnce(Y, &mut InstructionSequence<&mut u64, &Window<I>>) -> MixedResult<Z, E>,
    {
        self.parse_inner(locals_f, code_f)
            .with_location_context("code section entry", self.content.base())
    }

    /// Parses the contents of this code entry using the given closures.
    ///
    /// The first closure is given a [`Locals`](component::Locals) used to read the compressed
    /// local variable declarations.
    ///
    /// The second closure is given the output of the first closure, along with an
    /// [`InstructionSequence`] used to read the function *body*.
    #[inline]
    pub fn parse<Y, Z, L, C>(&self, locals_f: L, code_f: C) -> Parsed<Z>
    where
        L: FnOnce(&mut component::Locals<&mut u64, &Window<I>>) -> Parsed<Y>,
        C: FnOnce(Y, &mut InstructionSequence<&mut u64, &Window<I>>) -> Parsed<Z>,
    {
        self.parse_mixed::<core::convert::Infallible, Y, Z, _, _>(
            |locals| locals_f(locals).map_err(Into::into),
            |result, code| code_f(result, code).map_err(Into::into),
        )
        .map_err(Into::into)
    }
}

impl<I: Input> HasInput<I> for Code<I> {
    #[inline]
    fn input(&self) -> &I {
        self.content.input()
    }
}

impl<I: Input> HasInput<Window<I>> for Code<I> {
    #[inline]
    fn input(&self) -> &Window<I> {
        self.content()
    }
}

impl<'a, I: Input + 'a> BorrowInput<'a, I> for Code<I> {
    type Borrowed = Code<&'a I>;

    #[inline]
    fn borrow_input(&'a self) -> Code<&'a I> {
        Code {
            index: self.index,
            content: self.content.borrow_input(),
        }
    }
}

impl<'a, I: Clone + Input + 'a> CloneInput<'a, I> for Code<&'a I> {
    type Cloned = Code<I>;

    #[inline]
    fn clone_input(&self) -> Code<I> {
        Code {
            index: self.index,
            content: self.content.clone_input(),
        }
    }
}

impl<I: Input> Debug for Code<I> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut s = f.debug_struct("Func");
        let result = self.parse(
            |locals| Ok(s.field("locals", locals)),
            |s, body| {
                s.field("body", body);
                Ok(())
            },
        );

        if let Err(e) = result {
            s.field("error", &e);
        }

        s.finish()
    }
}

/// Represents the
/// [*code section*](https://webassembly.github.io/spec/core/binary/modules.html#code-section),
/// which corresponds to the
/// [**locals** and **body** fields](https://webassembly.github.io/spec/core/syntax/modules.html#syntax-func)
/// of each function in the
/// [**funcs** component](https://webassembly.github.io/spec/core/syntax/modules.html#syntax-func)
/// of a WebAssembly module.
#[derive(Clone, Copy)]
pub struct CodeSection<I: Input> {
    entries: Vector<u64, I>,
}

impl<I: Input> From<Vector<u64, I>> for CodeSection<I> {
    #[inline]
    fn from(entries: Vector<u64, I>) -> Self {
        Self { entries }
    }
}

impl<I: Input> CodeSection<I> {
    /// Uses the given [`Input`] to read the contents of the *code section* of a module, which
    /// begins at the given `offset`.
    #[inline]
    pub fn new(offset: u64, input: I) -> parser::Parsed<Self> {
        Vector::parse(offset, input)
            .context("at start of code section")
            .map(Self::from)
    }

    /// Gets the expected remaining number of entries in the *code section* that have yet to be
    /// parsed.
    #[inline]
    pub fn remaining_count(&self) -> u32 {
        self.entries.remaining_count()
    }

    /// Parses the next entry in the *code section*.
    pub fn parse(&mut self) -> Parsed<Option<Code<&I>>> {
        self.entries
            .advance_with_index(|index, offset, bytes| {
                let size = parser::leb128::u64(offset, bytes).context("code entry size")?;
                let content = Window::with_offset_and_length(bytes, *offset, size);

                crate::input::increment_offset(offset, size)
                    .context("unable to advance offset to read next code section entry")?;

                Parsed::Ok(Code { index, content })
            })
            .transpose()
            .context("within code section")
    }
}

impl<I: Input> HasInput<I> for CodeSection<I> {
    #[inline]
    fn input(&self) -> &I {
        self.entries.input()
    }
}

impl<'a, I: Input + 'a> BorrowInput<'a, I> for CodeSection<I> {
    type Borrowed = CodeSection<&'a I>;

    #[inline]
    fn borrow_input(&'a self) -> Self::Borrowed {
        self.entries.borrow_input().into()
    }
}

impl<'a, I: Clone + Input + 'a> CloneInput<'a, I> for CodeSection<&'a I> {
    type Cloned = CodeSection<I>;

    #[inline]
    fn clone_input(&self) -> Self::Cloned {
        self.entries.clone_input().into()
    }
}

impl<I: Clone + Input> Iterator for CodeSection<I> {
    type Item = Parsed<Code<I>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.parse() {
            Ok(None) => None,
            Err(e) => Some(Err(e)),
            Ok(Some(code)) => Some(Ok(code.clone_input())),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.entries.size_hint()
    }
}

impl<I: Clone + Input> core::iter::FusedIterator for CodeSection<I> {}

impl<I: Input> Debug for CodeSection<I> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.borrow_input()).finish()
    }
}
