use crate::bytes::Bytes;
use crate::component::{Code, CodeSection, FuncIdx, FunctionSection, TypeIdx};
use crate::parser::Result;

/// A WebAssembly function, defined in the
/// [**funcs** component](https://webassembly.github.io/spec/core/syntax/modules.html#syntax-func)
/// of a WebAssembly module.
#[derive(Clone, Copy)]
pub struct Func<C: Bytes> {
    index: FuncIdx,
    r#type: TypeIdx,
    code: Code<C>,
}

impl<C: Bytes> Func<C> {
    /// Gets the index used to refer to this function.
    #[inline]
    pub fn index(&self) -> FuncIdx {
        self.index
    }

    /// Gets an index into the *type section* that specifies the type of this function.
    #[inline]
    pub fn signature(&self) -> TypeIdx {
        self.r#type
    }

    /// Gets the local variables and body of this function.
    #[inline]
    pub fn code(&self) -> &Code<C> {
        &self.code
    }
}

impl<C: Bytes> core::fmt::Debug for Func<C> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Func")
            .field("type", &self.r#type)
            .field("code", &self.code)
            .finish()
    }
}

/// Represents the contents of the
/// [**funcs** component](https://webassembly.github.io/spec/core/syntax/modules.html#syntax-func)
/// of a WebAssembly module.
#[derive(Clone, Copy)]
pub struct FuncsComponent<T: Bytes, C: Bytes> {
    types: FunctionSection<T>,
    code: CodeSection<C>,
}

impl<T: Bytes, C: Bytes> FuncsComponent<T, C> {
    /// Creates a [`FuncsComponent`] from the given *function* and *code* sections.
    ///
    /// # Errors
    ///
    /// Returns an error if the length of both sections are not the same.
    pub fn new(types: FunctionSection<T>, code: CodeSection<C>) -> Result<Self> {
        if types.len() != code.len() {
            Err(crate::parser_bad_format!(
                "function section has {} entries, but code section has {} entries",
                types.len(),
                code.len()
            ))
        } else {
            Ok(Self { types, code })
        }
    }

    /// Parses the *function* and *code* sections to read the next function.
    pub fn parse(&mut self) -> Result<Option<Func<&C>>> {
        // Constructor ensures both sections have the same count
        match self.types.next() {
            None => Ok(None),
            Some(Err(e)) => Err(e),
            Some(Ok(r#type)) => {
                let index = FuncIdx::try_from(self.code.len() - 1)?;
                let code = self.code.parse()?.unwrap();
                Ok(Some(Func {
                    index,
                    r#type,
                    code,
                }))
            }
        }
    }
}

impl<T: Bytes, C: Bytes> core::fmt::Debug for FuncsComponent<T, C> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut funcs = FuncsComponent {
            types: self.types.borrowed(),
            code: self.code.borrowed(),
        };
        let mut list = f.debug_list();
        while let Some(func) = funcs.parse().transpose() {
            list.entry(&func);
        }
        list.finish()
    }
}
