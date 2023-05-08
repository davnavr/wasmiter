use crate::component::ValType;
use crate::parser::input::Input;
use crate::parser::{self, Decoder, Result, ResultExt};

/// Parser for a
/// [WebAssembly result type](https://webassembly.github.io/spec/core/binary/types.html#result-types).
pub type ResultType<I: Input> = parser::Vector<I, parser::SimpleParse<ValType>>;

/// Represents the
/// [**types** component](https://webassembly.github.io/spec/core/syntax/modules.html#types) of a
/// WebAssembly module, stored in and parsed from the
/// [*type section*](https://webassembly.github.io/spec/core/binary/modules.html#type-section).
#[derive(Debug)]
pub struct TypesComponent<I: Input> {
    count: usize,
    parser: Decoder<I>,
}

impl<I: Input> TypesComponent<I> {
    /// Uses a [`Decoder<I>`] to read the contents of the *type section* of a module.
    pub fn new(mut parser: Decoder<I>) -> Result<Self> {
        Ok(Self {
            count: parser.leb128_usize().context("type section count")?,
            parser,
        })
    }

    /// Gets the expected remaining number of types that have yet to be parsed.
    #[inline]
    pub fn count(&mut self) -> usize {
        self.count
    }

    /// Gets the next function type in the section. Returns `Ok(true)` if a type was parsed; or
    /// `Ok(false)` if there are no more types remaining.
    #[inline]
    pub fn next<P, R>(&mut self, parameter_types: P, result_types: R) -> Result<bool>
    where
        P: FnOnce(&mut ResultType<&mut I>) -> Result<()>,
        R: FnOnce(&mut ResultType<&mut I>) -> Result<()>,
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

    fn try_clone(&self) -> Result<TypesComponent<I::Fork>> {
        Ok(TypesComponent {
            count: self.count,
            parser: self.parser.fork()?,
        })
    }
}
