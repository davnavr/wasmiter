use crate::{
    bytes::Bytes,
    component,
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
pub struct Locals<O: Offset, B: Bytes> {
    offset: O,
    bytes: B,
    count: u32,
    current: Option<(NonZeroU32, ValType)>,
}

impl<O: Offset, B: Bytes> Locals<O, B> {
    pub(super) fn new(mut offset: O, bytes: B) -> parser::Result<Self> {
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

    /// Parses all local variable declarations.
    pub fn finish(mut self) -> parser::Result<O> {
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
