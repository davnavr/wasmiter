use crate::bytes::{Bytes, Window};
use crate::component::{self, ValType};
use crate::instruction_set::InstructionSequence;
use crate::parser::{self, Offset, Result, ResultExt};
use core::fmt::{Debug, Formatter};
use core::num::NonZeroU32;

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
    fn new(mut offset: O, bytes: B) -> Result<Self> {
        Ok(Self {
            count: parser::leb128::u32(offset.offset_mut(), &bytes)
                .context("locals declaration count")?,
            bytes,
            offset,
            current: None,
        })
    }

    fn load_next_group(&mut self) -> Result<Option<(NonZeroU32, ValType)>> {
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
    pub fn next_group(&mut self) -> Result<Option<(NonZeroU32, ValType)>> {
        self.load_next_group()?;
        Ok(self.current.take())
    }

    fn next_inner(&mut self) -> Result<Option<ValType>> {
        match self.load_next_group() {
            Ok(None) => Ok(None),
            Err(e) => Err(e),
            Ok(Some((count, ty))) => {
                self.current = NonZeroU32::new(count.get() - 1).map(|count| (count, ty));
                Ok(Some(ty))
            }
        }
    }

    fn finish(mut self) -> Result<O> {
        while self.next_group().transpose().is_some() {}
        Ok(self.offset)
    }
}

impl<O: Offset, B: Bytes> Iterator for Locals<O, B> {
    type Item = Result<ValType>;

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

/// Represents a
/// [`func`](https://webassembly.github.io/spec/core/binary/modules.html#code-section), an entry
/// in the
/// [*code section*](https://webassembly.github.io/spec/core/binary/modules.html#code-section).
///
/// To allow reading the code section in parallel and skipping of entries, a [`Func`] stores the
/// size, in bytes, of its contents.
#[derive(Clone, Copy)]
pub struct Func<B: Bytes> {
    content: Window<B>,
}

impl<B: Bytes> Func<B> {
    /// Gets the binary contents of this *code section* entry.
    pub fn content(&self) -> &Window<B> {
        &self.content
    }

    /// Reads the contents of this code entry.
    ///
    /// The first closure is given a [`Locals`] used to read the compressed local variable declarations.
    ///
    /// The second closure is given the output of the first closure, along with an
    /// [`InstructionSequence`] used to read the function *body*.
    pub fn read<Y, Z, L, C>(&self, locals_f: L, code_f: C) -> Result<Z>
    where
        L: FnOnce(&mut Locals<&mut u64, &Window<B>>) -> Result<Y>,
        C: FnOnce(Y, &mut InstructionSequence<&mut u64, &Window<B>>) -> Result<Z>,
    {
        let mut offset = self.content.base();
        let mut locals = Locals::new(&mut offset, &self.content)?;
        let code_arg = locals_f(&mut locals)?;
        locals.finish()?;

        let mut code = InstructionSequence::new(&mut offset, &self.content);
        let result = code_f(code_arg, &mut code)?;
        code.finish()?;
        Ok(result)
    }
}

impl<B: Bytes> Debug for Func<B> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut s = f.debug_struct("Func");
        let result = self.read(
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
pub struct CodeSection<B: Bytes> {
    count: u32,
    offset: u64,
    bytes: B,
}

impl<B: Bytes> CodeSection<B> {
    /// Uses the given [`Bytes`] to read the contents of the *code section* of a module, which
    /// begins at the given `offset`.
    #[inline]
    pub fn new(mut offset: u64, bytes: B) -> Result<Self> {
        Ok(Self {
            count: parser::leb128::u32(&mut offset, &bytes).context("code section count")?,
            bytes,
            offset,
        })
    }

    /// Parses the next entry in the **code section**.
    pub fn parse(&mut self) -> Result<Option<Func<&B>>> {
        if self.count == 0 {
            return Ok(None);
        }

        let result = parser::leb128::u32(&mut self.offset, &self.bytes).context("code entry size");
        match result {
            Ok(size) => {
                self.count -= 1;
                let content = Window::new(&self.bytes, self.offset, u64::from(size));
                Ok(Some(Func { content }))
            }
            Err(e) => {
                self.count = 0;
                Err(e)
            }
        }
    }
}

impl<B: Bytes> Debug for CodeSection<B> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut code = CodeSection {
            count: self.count,
            offset: self.offset,
            bytes: &self.bytes,
        };

        let mut list = f.debug_list();
        while let Some(func) = code.parse().transpose() {
            list.entry(&func);
            if func.is_err() {
                break;
            }
        }
        list.finish()
    }
}
