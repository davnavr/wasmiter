use crate::{
    component::ResultType,
    input::{BorrowInput, CloneInput, HasInput, Input},
    parser::{Parsed, ResultExt, Vector},
};

/// Represents the
/// [**types** component](https://webassembly.github.io/spec/core/syntax/modules.html#types) of a
/// WebAssembly module, stored in and parsed from the
/// [*type section*](https://webassembly.github.io/spec/core/binary/modules.html#type-section).
#[derive(Clone, Copy)]
pub struct TypesComponent<I: Input> {
    types: Vector<u64, I>,
}

impl<I: Input> From<Vector<u64, I>> for TypesComponent<I> {
    #[inline]
    fn from(types: Vector<u64, I>) -> Self {
        Self { types }
    }
}

impl<I: Input> TypesComponent<I> {
    /// Uses the given [`Input`] to read the contents of the *type section* of a module, starting
    /// at the specified `offset`.
    pub fn new(offset: u64, input: I) -> Parsed<Self> {
        Vector::parse(offset, input)
            .context("at start of type section")
            .map(Self::from)
    }

    /// Gets the expected remaining number of types that have yet to be parsed.
    #[inline]
    pub fn remaining_count(&self) -> u32 {
        self.types.remaining_count()
    }

    /// Parses the next function type in the section.
    #[inline]
    pub fn parse<Y, Z, P, R>(&mut self, parameter_types: P, result_types: R) -> Parsed<Option<Z>>
    where
        P: FnOnce(&mut ResultType<&mut u64, &I>) -> Parsed<Y>,
        R: FnOnce(Y, &mut ResultType<&mut u64, &I>) -> Parsed<Z>,
    {
        self.types
            .advance(|offset, bytes| {
                crate::component::func_type(offset, bytes, parameter_types, result_types)
            })
            .transpose()
    }
}

impl<I: Input> HasInput<I> for TypesComponent<I> {
    #[inline]
    fn input(&self) -> &I {
        self.types.input()
    }
}

impl<'a, I: Input + 'a> BorrowInput<'a, I> for TypesComponent<I> {
    type Borrowed = TypesComponent<&'a I>;

    #[inline]
    fn borrow_input(&'a self) -> Self::Borrowed {
        self.types.borrow_input().into()
    }
}

impl<I: Input> core::fmt::Debug for TypesComponent<I> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        struct FuncType<'a, I: Input> {
            parameters: ResultType<u64, &'a I>,
            results: ResultType<u64, &'a I>,
        }

        impl<I: Input> core::fmt::Debug for FuncType<'_, I> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct("FuncType")
                    .field("parameters", &self.parameters)
                    .field("results", &self.results)
                    .finish()
            }
        }

        let mut list = f.debug_list();
        let mut types = self.borrow_input();

        let empty_types = ResultType::empty_with_offset(0, self.types.input());
        let mut last_parameters = empty_types;
        let mut last_results = empty_types;

        loop {
            let result = types.parse(
                |parameter_types| {
                    last_parameters = parameter_types.clone_input();
                    Ok(())
                },
                |(), result_types| {
                    last_results = result_types.clone_input();
                    Ok(())
                },
            );

            match result {
                Ok(Some(())) => {
                    let parameters = core::mem::replace(&mut last_parameters, empty_types);
                    let results = core::mem::replace(&mut last_results, empty_types);
                    list.entry(&FuncType {
                        parameters,
                        results,
                    });
                }
                Ok(None) => break,
                Err(e) => {
                    list.entry(&Parsed::<()>::Err(e));
                    break;
                }
            }
        }

        list.finish()
    }
}
