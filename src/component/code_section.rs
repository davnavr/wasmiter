use crate::{
    bytes::{Bytes, Window},
    component,
    instruction_set::InstructionSequence,
    parser::{self, Offset, ResultExt as _, Vector},
    types::ValType,
};
use core::{
    fmt::{Debug, Formatter},
    num::NonZeroU32,
};

/// Represents the local declarations in the [*code section*](https://webassembly.github.io/spec/core/binary/modules.html#code-section),
/// which corresponds to the
/// [**locals** field](https://webassembly.github.io/spec/core/syntax/modules.html#syntax-func) of
/// each function in the
/// [**funcs** component](https://webassembly.github.io/spec/core/syntax/modules.html#syntax-func)
/// of a WebAssembly module.
pub struct Locals<O: Offset, B: Bytes> {
    offset: O,
    bytes: B,
    count: u32,
    current: Option<(NonZeroU32, ValType)>,
}

impl<O: Offset, B: Bytes> Locals<O, B> {
    fn new(mut offset: O, bytes: B) -> parser::Result<Self> {
        Ok(Self {
            count: parser::leb128::u32(offset.offset_mut(), &bytes)
                .context("locals declaration count")?,
            bytes,
            offset,
            current: None,
        })
    }

    fn load_next_group(&mut self) -> parser::Result<Option<(NonZeroU32, ValType)>> {
        if self.count == 0 {
            return Ok(None);
        }

        if let Some(existing) = self.current {
            Ok(Some(existing))
        } else {
            loop {
                self.count -= 1;

                let count = parser::leb128::u32(self.offset.offset_mut(), &self.bytes)
                    .context("local group count")?;

                if let Some(variable_count) = NonZeroU32::new(count) {
                    let variable_type = component::val_type(self.offset.offset_mut(), &self.bytes)
                        .context("local group type")?;

                    return Ok(Some(*self.current.insert((variable_count, variable_type))));
                } else {
                    continue;
                }
            }
        }
    }

    /// Gets the next group of local variable declarations. Returns a type, and the number of
    /// locals of that type.
    ///
    /// To save on size, locals of the same type can be grouped together.
    pub fn next_group(&mut self) -> parser::Result<Option<(NonZeroU32, ValType)>> {
        self.load_next_group()?;
        Ok(self.current.take())
    }

    fn next_inner(&mut self) -> parser::Result<Option<ValType>> {
        match self.load_next_group() {
            Ok(None) => Ok(None),
            Err(e) => Err(e),
            Ok(Some((count, ty))) => {
                self.current = NonZeroU32::new(count.get() - 1).map(|count| (count, ty));
                Ok(Some(ty))
            }
        }
    }

    fn finish(mut self) -> parser::Result<O> {
        while self.next_group().transpose().is_some() {}
        Ok(self.offset)
    }
}

impl<O: Offset, B: Bytes> Iterator for Locals<O, B> {
    type Item = parser::Result<ValType>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.next_inner().transpose()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (
            self.current
                .and_then(|(count, _)| usize::try_from(count.get()).ok())
                .unwrap_or(0),
            None,
        )
    }
}

impl<O: Offset, B: Bytes> Debug for Locals<O, B> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let borrowed = Locals {
            offset: self.offset.offset(),
            bytes: &self.bytes,
            count: self.count,
            current: self.current,
        };

        f.debug_list().entries(borrowed).finish()
    }
}

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
pub struct Code<B: Bytes> {
    index: u32,
    content: Window<B>,
}

impl<B: Bytes> Code<B> {
    /// The index of this *code section* entry.
    #[inline]
    pub fn index(&self) -> u32 {
        self.index
    }

    /// Gets the binary contents of this *code section* entry.
    #[inline]
    pub fn content(&self) -> &Window<B> {
        &self.content
    }

    /// Reads the contents of this code entry.
    ///
    /// The first closure is given a [`Locals`] used to read the compressed local variable declarations.
    ///
    /// The second closure is given the output of the first closure, along with an
    /// [`InstructionSequence`] used to read the function *body*.
    pub fn read<Y, Z, E, L, C>(&self, locals_f: L, code_f: C) -> Result<Z, E>
    where
        E: From<parser::Error>,
        L: FnOnce(&mut Locals<&mut u64, &Window<B>>) -> Result<Y, E>,
        C: FnOnce(Y, &mut InstructionSequence<&mut u64, &Window<B>>) -> Result<Z, E>,
    {
        let mut offset = self.content.base();
        let mut locals = Locals::new(&mut offset, &self.content)?;
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

impl<B: Bytes + Clone> Code<&B> {
    /// Clones the underlying [`Bytes`] of this *code section* entry.
    pub fn cloned(&self) -> Code<B> {
        Code {
            index: self.index,
            content: self.content.cloned(),
        }
    }
}

impl<B: Bytes> Debug for Code<B> {
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
pub struct CodeSection<B: Bytes> {
    entries: Vector<u64, B>,
}

impl<B: Bytes> From<Vector<u64, B>> for CodeSection<B> {
    #[inline]
    fn from(entries: Vector<u64, B>) -> Self {
        Self { entries }
    }
}

impl<B: Bytes> CodeSection<B> {
    /// Uses the given [`Bytes`] to read the contents of the *code section* of a module, which
    /// begins at the given `offset`.
    #[inline]
    pub fn new(offset: u64, bytes: B) -> parser::Result<Self> {
        Vector::parse(offset, bytes)
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
    pub fn parse(&mut self) -> parser::Result<Option<Code<&B>>> {
        self.entries
            .advance_with_index(|index, offset, bytes| {
                let size = parser::leb128::u64(offset, bytes).context("code entry size")?;
                let content = Window::new(bytes, *offset, size);

                *offset = offset.checked_add(size).ok_or_else(|| {
                    crate::parser_bad_input!(
                        crate::bytes::offset_overflowed(),
                        "unable to advance offset to read next code section entry"
                    )
                })?;

                parser::Result::Ok(Code { index, content })
            })
            .transpose()
            .context("within code section")
    }

    #[inline]
    pub(super) fn borrowed(&self) -> CodeSection<&B> {
        self.entries.borrowed().into()
    }
}

impl<B: Clone + Bytes> Iterator for CodeSection<B> {
    type Item = parser::Result<Code<B>>;

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

impl<B: Clone + Bytes> core::iter::FusedIterator for CodeSection<B> {}

impl<B: Bytes> Debug for CodeSection<B> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.borrowed()).finish()
    }
}
