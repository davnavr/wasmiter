use crate::{
    component::{Code, CodeSection, FunctionSection},
    index::TypeIdx,
    input::Input,
    parser::{self, Result},
};

/// A WebAssembly function, defined in the
/// [**funcs** component](https://webassembly.github.io/spec/core/syntax/modules.html#syntax-func)
/// of a WebAssembly module.
#[derive(Clone, Copy)]
pub struct Func<C: Input> {
    r#type: TypeIdx,
    code: Code<C>,
}

impl<C: Input> Func<C> {
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

impl<C: Input + Clone> Func<&C> {
    /// Returns a version of the [`Func`] with the code contents cloned.
    #[inline]
    pub fn cloned(&self) -> Func<C> {
        Func {
            r#type: self.r#type,
            code: self.code.cloned(),
        }
    }
}

impl<C: Input> core::fmt::Debug for Func<C> {
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
pub struct FuncsComponent<T: Input, C: Input> {
    types: FunctionSection<T>,
    code: CodeSection<C>,
}

impl<T: Input, C: Input> FuncsComponent<T, C> {
    /// Creates a [`FuncsComponent`] from the given *function* and *code* sections.
    ///
    /// # Errors
    ///
    /// Returns an error if the length of both sections are not the same.
    pub fn new(types: FunctionSection<T>, code: CodeSection<C>) -> Result<Self> {
        let type_count = types.remaining_count();
        let code_count = code.remaining_count();
        if type_count != code_count {
            #[cold]
            #[inline(never)]
            fn section_count_mismatch(type_count: u32, code_count: u32) -> parser::Error {
                parser::Error::new(parser::ErrorKind::InvalidFormat).with_context(parser::Context::from_closure(move |f| write!(f, "function section has {type_count} entries, but code section has {code_count} entries")))
            }

            Err(section_count_mismatch(type_count, code_count))
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

    pub(crate) fn borrowed(&self) -> FuncsComponent<&T, &C> {
        FuncsComponent {
            types: self.types.borrowed(),
            code: self.code.borrowed(),
        }
    }
}

impl<T: Clone + Input, C: Clone + Input> Iterator for FuncsComponent<T, C> {
    type Item = Result<Func<C>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.parse()
            .map(|result| result.map(|i| i.cloned()))
            .transpose()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let types = self.types.size_hint();
        let code = self.code.size_hint();
        (
            core::cmp::min(types.0, code.0),
            types.1.and_then(|t| code.1.map(|c| core::cmp::min(t, c))),
        )
    }
}

impl<T: Clone + Input, C: Clone + Input> core::iter::FusedIterator for FuncsComponent<T, C> {}

impl<T: Input, C: Input> core::fmt::Debug for FuncsComponent<T, C> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.borrowed()).finish()
    }
}
