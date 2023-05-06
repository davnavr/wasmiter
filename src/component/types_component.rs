use crate::allocator::{self, Allocator, Vector};
use crate::component::{type_parser, FuncType, ValType};
use crate::parser::input::Input;
use crate::parser::{Parser, Result, ResultExt};

fn parse_result_type(
    parser: &mut Parser<impl Input>,
    count: usize,
    buffer: &mut impl Vector<ValType>,
) -> Result<()> {
    buffer.reserve_exact(count);
    for _ in 0..count {
        buffer.push(type_parser::parse_val_type(parser)?);
    }
    Ok(())
}

/// Represents the
/// [**types** component](https://webassembly.github.io/spec/core/syntax/modules.html#types) of a
/// WebAssembly module, stored in and parsed from the
/// [*type section*](https://webassembly.github.io/spec/core/binary/modules.html#type-section).
pub struct TypesComponent<I: Input, A: Allocator> {
    count: usize,
    parser: Parser<I>,
    buffer: A::Vec<ValType>,
    allocator: A,
}

impl<I: Input, A: Allocator> TypesComponent<I, A> {
    /// Uses a [`Parser<I>`] to read the contents of the *type section* of a module, using the
    /// [`Allocator`] as a buffer when reading types.
    pub fn with_allocator(mut parser: Parser<I>, allocator: A) -> Result<Self> {
        Ok(Self {
            count: parser.leb128_usize().context("type section count")?,
            parser,
            buffer: allocator.allocate_vector(),
            allocator,
        })
    }

    /// Gets the expected remaining number of types that have yet to be parsed.
    #[inline]
    pub fn count(&mut self) -> usize {
        self.count
    }

    fn parse_next(&mut self) -> Result<Option<FuncType<A::Vec<ValType>>>> {
        if self.count == 0 {
            return Ok(None);
        }

        let mut tag_byte = 0u8;
        self.parser
            .bytes_exact(core::slice::from_mut(&mut tag_byte))
            .context("functype tag")?;

        if tag_byte != 0x60 {
            return Err(crate::parser_bad_format!(
                "expected functype (0x60) but got {tag_byte:#02X}"
            ));
        }

        self.buffer.clear();

        let parameter_count = self.parser.leb128_usize().context("parameter type count")?;
        parse_result_type(&mut self.parser, parameter_count, &mut self.buffer)
            .context("parameter types")?;

        let result_count = self.parser.leb128_usize().context("result type count")?;
        parse_result_type(&mut self.parser, result_count, &mut self.buffer)
            .context("result types")?;

        let func_type =
            FuncType::from_slice_in(parameter_count, self.buffer.as_ref(), &self.allocator);

        self.count -= 1;
        Ok(Some(func_type))
    }
}

#[cfg(feature = "alloc")]
impl<I: Input> TypesComponent<I, allocator::Global> {
    /// Uses a [`Parser<I>`] to read the contents of the *type section* of a module.
    #[inline]
    pub fn new(parser: Parser<I>) -> Result<Self> {
        Self::with_allocator(parser, allocator::Global)
    }
}

impl<I: Input, A: Allocator> core::iter::Iterator for TypesComponent<I, A> {
    type Item = Result<FuncType<A::Vec<ValType>>>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.parse_next().transpose()
    }

    #[inline]
    fn count(self) -> usize {
        self.count
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.count, Some(self.count))
    }
}

// count is correct, since errors are returned if there are too few elements
impl<I: Input, A: Allocator> core::iter::ExactSizeIterator for TypesComponent<I, A> {
    fn len(&self) -> usize {
        self.count
    }
}

impl<I: Input + core::fmt::Debug, A: Allocator> core::fmt::Debug for TypesComponent<I, A> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TypesComponent")
            .field("count", &self.count)
            .field("parser", &self.parser)
            .finish_non_exhaustive()
    }
}
