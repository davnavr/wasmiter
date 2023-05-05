use crate::allocator::{self, Allocator, Buffer, OwnOrRef, StringPool};
use crate::component::{self, Index};
use crate::parser::input::Input;
use crate::parser::{Parser, Result, ResultExt};
use core::fmt::Debug;

fn cached_module_name(name: &str) -> Option<&'static str> {
    macro_rules! names {
        ($($name:literal,)*) => {
            Some(match name {
                $($name => $name,)*
                _ => return None,
            })
        };
    }

    names!["env", "wasi_snapshot_preview1",]
}

/// Describes what kind of entity is specified by an [`Import`].
#[derive(Clone, Debug)]
pub enum ImportKind {
    /// An imported function with the specified signature.
    Function(component::TypeIdx),
    // /// An imported table with the specified limits and element type.
    // Table(component::TableType),
    // /// An imported table with the specified limits.
    // Memory(component::Limits),
    // /// An imported global with the specified type.
    // Global(component::GlobalType),
}

/// Represents a
/// [WebAssembly import](https://webassembly.github.io/spec/core/binary/modules.html#import-section).
#[derive(Clone)]
pub struct Import<S: AsRef<str>> {
    module: OwnOrRef<'static, str, S>,
    name: S,
    kind: ImportKind,
}

impl<S: AsRef<str>> Debug for Import<S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Import")
            .field("module", &self.module.as_ref())
            .field("name", &self.name.as_ref())
            .field("kind", &self.kind)
            .finish()
    }
}

/// Represents the
/// [**imports** component](https://webassembly.github.io/spec/core/syntax/modules.html#imports) of
/// a WebAssembly module, stored in and parsed from the
/// [*imports section*](https://webassembly.github.io/spec/core/binary/modules.html#import-section).
pub struct ImportsComponent<I, S, A>
where
    I: Input,
    S: StringPool,
    A: Allocator<String = S::Interned>,
{
    count: usize,
    parser: Parser<I>,
    string_cache: S,
    name_buffer: A::Buf,
    allocator: A,
}

impl<I, S, A> ImportsComponent<I, S, A>
where
    I: Input,
    S: StringPool,
    A: Allocator<String = S::Interned>,
{
    /// Uses a [`Parser<I>`] to read the contents of the *imports section* of a module, with the
    /// given [`StringPool`] used to intern module names and [`Allocator`].
    pub fn with_string_cache_and_buffer(
        mut parser: Parser<I>,
        string_cache: S,
        allocator: A,
    ) -> Result<Self> {
        Ok(Self {
            count: parser.leb128_usize().context("import section count")?,
            parser,
            string_cache,
            name_buffer: allocator.allocate_buffer(),
            allocator,
        })
    }

    fn parse_next(&mut self) -> Result<Option<Import<S::Interned>>> {
        if self.count == 0 {
            return Ok(None);
        }

        let module_name = self
            .parser
            .name(&mut self.name_buffer)
            .context("module name")?;

        let module = if let Some(cached) = cached_module_name(module_name) {
            OwnOrRef::Reference(cached)
        } else {
            OwnOrRef::Owned(self.string_cache.get(module_name))
        };

        let name = self.string_cache.get(
            self.parser
                .name(&mut self.name_buffer)
                .context("import name")?,
        );

        let mut kind_tag = 0u8;
        self.parser
            .bytes_exact(core::slice::from_mut(&mut kind_tag))
            .context("import kind")?;

        let kind = match kind_tag {
            0 => ImportKind::Function(
                component::TypeIdx::parse(&mut self.parser).context("function import type")?,
            ),
            _ => {
                return Err(crate::parser_bad_format!(
                    "{kind_tag:#02X} is not a known import kind"
                ))
            }
        };

        let import = Import { module, name, kind };

        self.count -= 1;
        Ok(Some(import))
    }
}

#[cfg(feature = "alloc")]
impl<I: Input> ImportsComponent<I, allocator::FakeStringPool, allocator::Global> {
    /// Uses a [`Parser<I>`] to read the contents of the *imports section* of a module.
    pub fn new(parser: Parser<I>) -> Result<Self> {
        Self::with_string_cache_and_buffer(parser, allocator::FakeStringPool, allocator::Global)
    }
}

impl<I, S, A> core::iter::Iterator for ImportsComponent<I, S, A>
where
    I: Input,
    S: StringPool,
    A: Allocator<String = S::Interned>,
{
    type Item = Result<Import<S::Interned>>;

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
impl<I, S, A> core::iter::ExactSizeIterator for ImportsComponent<I, S, A>
where
    I: Input,
    S: StringPool,
    A: Allocator<String = S::Interned>,
{
    fn len(&self) -> usize {
        self.count
    }
}

impl<I, S, A> Debug for ImportsComponent<I, S, A>
where
    I: Input + Debug,
    S: StringPool,
    A: Allocator<String = S::Interned>,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TypesComponent")
            .field("count", &self.count)
            .field("parser", &self.parser)
            .finish_non_exhaustive()
    }
}
