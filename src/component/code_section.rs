use crate::{
    component,
    input::{Input, Window},
    instruction_set::InstructionSequence,
    parser::{self, ResultExt as _, Vector},
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

    /// Reads the contents of this code entry.
    ///
    /// The first closure is given a [`Locals`](component::Locals) used to read the compressed
    /// local variable declarations.
    ///
    /// The second closure is given the output of the first closure, along with an
    /// [`InstructionSequence`] used to read the function *body*.
    pub fn read<Y, Z, E, L, C>(&self, locals_f: L, code_f: C) -> Result<Z, E>
    where
        E: From<parser::Error>,
        L: FnOnce(&mut component::Locals<&mut u64, &Window<I>>) -> Result<Y, E>,
        C: FnOnce(Y, &mut InstructionSequence<&mut u64, &Window<I>>) -> Result<Z, E>,
    {
        let mut offset = self.content.base();
        let mut locals = component::Locals::new(&mut offset, &self.content)?;
        let code_arg = locals_f(&mut locals)?;
        locals.finish()?;

        let mut code = InstructionSequence::new(&mut offset, &self.content);
        let result = code_f(code_arg, &mut code)?;

        let (_, final_offset) = code.finish()?;
        let final_length = *final_offset - self.content.base();
        if final_length != self.content.length() {
            return Err(crate::parser_bad_format_at_offset!(
                "file" @ offset,
                "expected code entry content to have a length of {} bytes, but got {final_length}",
                self.content.length()
            )
            .into());
        }

        Ok(result)
    }
}

impl<I: Input + Clone> Code<&I> {
    /// Clones the underlying [`Bytes`] of this *code section* entry.
    pub fn cloned(&self) -> Code<I> {
        Code {
            index: self.index,
            content: (&self.content).into(),
        }
    }
}

impl<I: Input> Debug for Code<I> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut s = f.debug_struct("Func");
        let result = self.read(
            |locals| Ok(s.field("locals", locals)),
            |s, body| {
                s.field("body", body);
                parser::Result::Ok(())
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
    pub fn new(offset: u64, input: I) -> parser::Result<Self> {
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
    pub fn parse(&mut self) -> parser::Result<Option<Code<&I>>> {
        self.entries
            .advance_with_index(|index, offset, bytes| {
                let size = parser::leb128::u64(offset, bytes).context("code entry size")?;
                let content = Window::with_offset_and_length(bytes, *offset, size);

                *offset = offset.checked_add(size).ok_or_else(|| {
                    crate::parser_bad_input!(
                        crate::input::offset_overflowed(*offset),
                        "unable to advance offset to read next code section entry"
                    )
                })?;

                parser::Result::Ok(Code { index, content })
            })
            .transpose()
            .context("within code section")
    }

    #[inline]
    pub(super) fn borrowed(&self) -> CodeSection<&I> {
        self.entries.borrowed().into()
    }
}

impl<I: Clone + Input> Iterator for CodeSection<I> {
    type Item = parser::Result<Code<I>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.parse() {
            Ok(None) => None,
            Err(e) => Some(Err(e)),
            Ok(Some(code)) => Some(Ok(code.cloned())),
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
        f.debug_list().entries(self.borrowed()).finish()
    }
}
