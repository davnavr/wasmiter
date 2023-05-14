use crate::bytes::Bytes;
use crate::component::ValType;
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

        let result = self.parser.func_type(parameter_types, result_types);

        if result.is_ok() {
            self.count -= 1;
        } else {
            self.count = 0;
        }

        result.map(|()| true)
    }

    fn by_ref(&self) -> TypesComponent<&B> {
        TypesComponent {
            count: self.count,
            offset: self.offset,
            bytes: &self.bytes,
        }
    }
}

#[cfg(feature = "alloc")]
fn allocate_result_type<B: Bytes>(
    vector: &mut alloc::vec::Vec<ValType>,
) -> impl FnOnce(&mut ResultType<&mut u64, &B>) -> Result<()> + '_ {
    |parser| {
        vector.reserve_exact(usize::try_from(parser.len()).unwrap_or_default());
        for ty in parser {
            vector.push(ty?);
        }
        Ok(())
    }
}

#[cfg(feature = "alloc")]
impl<B: Bytes> Iterator for TypesComponent<B> {
    type Item = Result<(alloc::vec::Vec<ValType>, alloc::vec::Vec<ValType>)>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut parameters = alloc::vec::Vec::new();
        let mut results = alloc::vec::Vec::new();
        let more = self.next(
            allocate_result_type(&mut parameters),
            allocate_result_type(&mut results),
        );
        match more {
            Ok(true) => Some(Ok((parameters, results))),
            Ok(false) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

#[cfg(feature = "alloc")]
impl<B: Bytes> core::fmt::Debug for TypesComponent<B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut list = f.debug_list();
        match self.try_clone() {
            Ok(fork) => list.entries(fork),
            Err(e) => list.entry(&Result::<()>::Err(e)),
        }
        .finish()
    }
}
