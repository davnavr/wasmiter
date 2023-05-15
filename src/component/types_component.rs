use crate::bytes::Bytes;
use crate::component::{self, ValType};
use crate::parser::{self, Result, ResultExt};

/// Parser for a
/// [WebAssembly result type](https://webassembly.github.io/spec/core/binary/types.html#result-types).
pub type ResultType<O, B> = parser::Vector<O, B, parser::SimpleParse<ValType>>;

/// Represents the
/// [**types** component](https://webassembly.github.io/spec/core/syntax/modules.html#types) of a
/// WebAssembly module, stored in and parsed from the
/// [*type section*](https://webassembly.github.io/spec/core/binary/modules.html#type-section).
#[derive(Clone, Copy)]
#[cfg_attr(not(feature = "alloc"), derive(Debug))]
pub struct TypesComponent<B: Bytes> {
    count: usize,
    offset: u64,
    bytes: B,
}

impl<B: Bytes> TypesComponent<B> {
    /// Uses the given [`Bytes`] to read the contents of the *type section* of a module, starting
    /// at the specified `offset`.
    pub fn new(mut offset: u64, bytes: B) -> Result<Self> {
        Ok(Self {
            count: parser::leb128::usize(&mut offset, &bytes).context("type section count")?,
            bytes,
            offset,
        })
    }

    /// Gets the expected remaining number of types that have yet to be parsed.
    #[inline]
    pub fn len(&self) -> usize {
        self.count
    }

    /// Returns a value indicating if the type section is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Gets the next function type in the section. Returns `Ok(true)` if a type was parsed; or
    /// `Ok(false)` if there are no more types remaining.
    #[inline]
    pub fn next<P, R>(&mut self, parameter_types: P, result_types: R) -> Result<bool>
    where
        P: FnOnce(&mut ResultType<&mut u64, &B>) -> Result<()>,
        R: FnOnce(&mut ResultType<&mut u64, &B>) -> Result<()>,
    {
        if self.count == 0 {
            return Ok(false);
        }

        let result =
            component::func_type(&mut self.offset, &self.bytes, parameter_types, result_types);

        if result.is_ok() {
            self.count -= 1;
        } else {
            self.count = 0;
        }

        result.map(|()| true)
    }
}

struct FuncType<'a, B: Bytes> {
    parameters: ResultType<u64, &'a B>,
    results: ResultType<u64, &'a B>,
}

impl<B: Bytes> core::fmt::Debug for FuncType<'_, B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FuncType")
            .field("parameters", &self.parameters)
            .field("results", &self.results)
            .finish()
    }
}

impl<B: Bytes> core::fmt::Debug for TypesComponent<B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut list = f.debug_list();
        let mut types = TypesComponent {
            count: self.count,
            offset: self.offset,
            bytes: &self.bytes,
        };

        let empty_types = ResultType::empty(&self.bytes, Default::default());
        let mut last_parameters = empty_types;
        let mut last_results = empty_types;

        loop {
            let result = types.next(
                |parameter_types| {
                    last_parameters = parameter_types.dereferenced();
                    Ok(())
                },
                |result_types| {
                    last_results = result_types.dereferenced();
                    Ok(())
                },
            );

            match result {
                Ok(true) => {
                    let parameters = core::mem::replace(&mut last_parameters, empty_types);
                    let results = core::mem::replace(&mut last_results, empty_types);
                    list.entry(&FuncType {
                        parameters,
                        results,
                    });
                }
                Ok(false) => break,
                Err(e) => {
                    list.entry(&Result::<()>::Err(e));
                    break;
                }
            }
        }

        list.finish()
    }
}
