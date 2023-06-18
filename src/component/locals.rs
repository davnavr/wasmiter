use crate::{
    component,
    input::{BorrowInput, HasInput, Input},
    parser::{self, Offset, ResultExt as _},
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
pub struct Locals<O: Offset, I: Input> {
    offset: O,
    input: I,
    count: u32,
    current: Option<(NonZeroU32, ValType)>,
}

impl<O: Offset, I: Input> Locals<O, I> {
    pub(super) fn new(mut offset: O, input: I) -> parser::Parsed<Self> {
        Ok(Self {
            count: parser::leb128::u32(offset.offset_mut(), &input)
                .context("locals declaration count")?,
            input,
            offset,
            current: None,
        })
    }

    fn load_next_group(&mut self) -> parser::Parsed<Option<(NonZeroU32, ValType)>> {
        if self.count == 0 {
            return Ok(None);
        }

        if let Some(existing) = self.current {
            Ok(Some(existing))
        } else {
            loop {
                self.count -= 1;

                let count = parser::leb128::u32(self.offset.offset_mut(), &self.input)
                    .context("local group count")?;

                if let Some(variable_count) = NonZeroU32::new(count) {
                    let variable_type = component::val_type(self.offset.offset_mut(), &self.input)
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
    pub fn next_group(&mut self) -> parser::Parsed<Option<(NonZeroU32, ValType)>> {
        self.load_next_group()?;
        Ok(self.current.take())
    }

    fn next_inner(&mut self) -> parser::Parsed<Option<ValType>> {
        match self.load_next_group() {
            Ok(None) => Ok(None),
            Err(e) => Err(e),
            Ok(Some((count, ty))) => {
                self.current = NonZeroU32::new(count.get() - 1).map(|count| (count, ty));
                Ok(Some(ty))
            }
        }
    }

    /// Parses all local variable declarations.
    pub fn finish(mut self) -> parser::Parsed<O> {
        while self.next_group().transpose().is_some() {}
        Ok(self.offset)
    }
}

impl<O: Offset, I: Input> Iterator for Locals<O, I> {
    type Item = parser::Parsed<ValType>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.next_inner().transpose()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let min = self
            .current
            .and_then(|(count, _)| usize::try_from(count.get()).ok())
            .unwrap_or(0);

        (
            min,
            usize::try_from(self.count)
                .ok()
                .and_then(|count| min.checked_add(count)),
        )
    }
}

impl<O: Offset, I: Input> HasInput<I> for Locals<O, I> {
    #[inline]
    fn input(&self) -> &I {
        &self.input
    }
}

impl<'a, O: Offset, I: Input + 'a> BorrowInput<'a, I> for Locals<O, I> {
    type Borrowed = Locals<u64, &'a I>;

    #[inline]
    fn borrow_input(&'a self) -> Self::Borrowed {
        Locals {
            offset: self.offset.offset(),
            input: &self.input,
            count: self.count,
            current: self.current,
        }
    }
}

impl<O: Offset, I: Input> Debug for Locals<O, I> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.borrow_input()).finish()
    }
}
