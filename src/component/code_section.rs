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

    fn next_inner<Y, Z, L, C>(&mut self, locals_f: L, code_f: C) -> Result<Z>
    where
        L: FnOnce(&mut Locals<&mut u64, &Window<&B>>) -> Result<Y>,
        C: FnOnce(Y, &mut InstructionSequence<&mut u64, Window<&B>>) -> Result<Z>,
    {
        // Get size for data after size field
        let size = parser::leb128::u32(&mut self.offset, &self.bytes).context("code entry size")?;
        let window = Window::new(&self.bytes, self.offset, u64::from(size));

        let mut locals = Locals::new(&mut self.offset, &window)?;
        let code_arg = locals_f(&mut locals)?;
        locals.finish()?;

        let mut code = InstructionSequence::new(&mut self.offset, window);
        let result = code_f(code_arg, &mut code)?;
        code.finish()?;
        Ok(result)
    }

    /// Parses the next entry in the **code section**.
    pub fn next<Y, Z, L, C>(&mut self, locals_f: L, code_f: C) -> Result<Option<Z>>
    where
        L: FnOnce(&mut Locals<&mut u64, &Window<&B>>) -> Result<Y>,
        C: FnOnce(Y, &mut InstructionSequence<&mut u64, Window<&B>>) -> Result<Z>,
    {
        if self.count == 0 {
            return Ok(None);
        }

        let result = self.next_inner(locals_f, code_f);

        if result.is_ok() {
            self.count -= 1;
        } else {
            self.count = 0;
        }

        result.map(Some)
    }
}

struct Func<'a, 'b, 'c, 'd, B: Bytes> {
    locals: Locals<u64, &'a B>,
    code: &'b mut InstructionSequence<&'c mut u64, Window<&'d &'a B>>,
}

impl<B: Bytes> Debug for Func<'_, '_, '_, '_, B> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Func")
            .field("locals", &self.locals)
            .field("code", self.code)
            .finish()
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
        while code.count > 0 {
            let result = code.next(
                |locals| {
                    Ok(Locals {
                        offset: locals.offset.offset(),
                        bytes: &self.bytes,
                        count: locals.count,
                        current: locals.current,
                    })
                },
                |locals, code| {
                    list.entry(&Func { locals, code });
                    Ok(())
                },
            );

            if let Err(e) = result {
                list.entry(&Result::<()>::Err(e));
                break;
            }
        }
        list.finish()
    }
}
