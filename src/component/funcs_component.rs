use crate::{
    component::{Code, CodeSection, FunctionSection},
    index::TypeIdx,
    input::{BorrowInput, CloneInput, HasInput, Input, Window},
    parser::{self, Parsed},
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

impl<I: Input> HasInput<I> for Func<I> {
    #[inline]
    fn input(&self) -> &I {
        self.code.input()
    }
}

impl<I: Input> HasInput<Window<I>> for Func<I> {
    #[inline]
    fn input(&self) -> &Window<I> {
        self.code.input()
    }
}

impl<'a, I: Input + 'a> BorrowInput<'a, I> for Func<I> {
    type Borrowed = Func<&'a I>;

    #[inline]
    fn borrow_input(&'a self) -> Func<&'a I> {
        Func {
            r#type: self.r#type,
            code: self.code.borrow_input(),
        }
    }
}

impl<'a, I: Clone + Input + 'a> CloneInput<'a, I> for Func<&'a I> {
    type Cloned = Func<I>;

    #[inline]
    fn clone_input(&self) -> Func<I> {
        Func {
            r#type: self.r#type,
            code: self.code.clone_input(),
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
    pub fn new(types: FunctionSection<T>, code: CodeSection<C>) -> Parsed<Self> {
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
    pub fn parse(&mut self) -> Parsed<Option<Func<&C>>> {
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

impl<T: Input, C: Input> HasInput<C> for FuncsComponent<T, C> {
    /// Returns the [`Input`] corresponding to the **code section**.
    #[inline]
    fn input(&self) -> &C {
        self.code.input()
    }
}

impl<'a, T: Input + 'a, C: Input + 'a> BorrowInput<'a, C> for FuncsComponent<T, C> {
    type Borrowed = FuncsComponent<&'a T, &'a C>;

    #[inline]
    fn borrow_input(&'a self) -> Self::Borrowed {
        FuncsComponent {
            types: self.types.borrow_input(),
            code: self.code.borrow_input(),
        }
    }
}

impl<'a, T: Clone + Input + 'a, C: Clone + Input + 'a> CloneInput<'a, C>
    for FuncsComponent<&'a T, &'a C>
{
    type Cloned = FuncsComponent<T, C>;

    #[inline]
    fn clone_input(&self) -> Self::Cloned {
        FuncsComponent {
            types: self.types.clone_input(),
            code: self.code.clone_input(),
        }
    }
}

impl<T: Input, C: Clone + Input> Iterator for FuncsComponent<T, C> {
    type Item = Parsed<Func<C>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.parse()
            .map(|result| result.map(|i| i.clone_input()))
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

impl<T: Input, C: Clone + Input> core::iter::FusedIterator for FuncsComponent<T, C> {}

impl<T: Input, C: Input> core::fmt::Debug for FuncsComponent<T, C> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.borrow_input()).finish()
    }
}
