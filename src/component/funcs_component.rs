use crate::bytes::Bytes;
use crate::component::{Code, CodeSection, FunctionSection};
use crate::index::TypeIdx;
use crate::parser::Result;

/// A WebAssembly function, defined in the
/// [**funcs** component](https://webassembly.github.io/spec/core/syntax/modules.html#syntax-func)
/// of a WebAssembly module.
#[derive(Clone, Copy)]
pub struct Func<C: Bytes> {
    r#type: TypeIdx,
    code: Code<C>,
}

impl<C: Bytes> Func<C> {
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

    /// Returns the function body.
    #[inline]
    pub fn into_code(self) -> Code<C> {
        self.code
    }
}

impl<C: Bytes + Clone> Func<&C> {
    /// Returns a version of the [`Func`] with the code contents cloned.
    #[inline]
    pub fn cloned(&self) -> Func<C> {
        Func {
            r#type: self.r#type,
            code: self.code.cloned(),
        }
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
                let code = self.code.parse()?.unwrap();
                Ok(Some(Func { r#type, code }))
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
