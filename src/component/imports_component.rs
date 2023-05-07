use crate::allocator::{Buffer, OwnOrRef, StringPool};
use crate::component;
use crate::parser::{input::Input, Decoder, Result, ResultExt};
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
    /// An imported table with the specified limits and element type.
    Table(component::TableType),
    /// An imported table with the specified limits.
    Memory(component::MemType),
    /// An imported global with the specified type.
    Global(component::GlobalType),
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
pub struct ImportsComponent<I: Input, S: StringPool, B: Buffer> {
    count: usize,
    parser: Decoder<I>,
    string_cache: S,
    name_buffer: B,
}

impl<I: Input, S: StringPool, B: Buffer> ImportsComponent<I, S, B> {
    /// Uses a [`Decoder<I>`] to read the contents of the *imports section* of a module, with the
    /// given [`StringPool`] used to intern module names and [`Buffer`].
    pub fn with_string_cache_and_buffer(
        mut parser: Decoder<I>,
        name_buffer: B,
        string_cache: S,
    ) -> Result<Self> {
        Ok(Self {
            count: parser.leb128_usize().context("import section count")?,
            parser,
            string_cache,
            name_buffer,
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

        let kind = match self.parser.one_byte_exact().context("import kind")? {
            0 => ImportKind::Function(self.parser.index().context("function import type")?),
            1 => ImportKind::Table(self.parser.table_type().context("table import type")?),
            2 => ImportKind::Memory(self.parser.mem_type().context("memory import type")?),
            3 => ImportKind::Global(self.parser.global_type().context("global import type")?),
            bad => {
                return Err(crate::parser_bad_format!(
                    "{bad:#02X} is not a known import kind"
                ))
            }
        };

        let import = Import { module, name, kind };

        self.count -= 1;
        Ok(Some(import))
    }
}

#[cfg(feature = "alloc")]
impl<I: Input> ImportsComponent<I, crate::allocator::FakeStringPool, alloc::vec::Vec<u8>> {
    /// Uses a [`Decoder<I>`] to read the contents of the *imports section* of a module.
    pub fn new(parser: Decoder<I>) -> Result<Self> {
        Self::with_string_cache_and_buffer(parser, Default::default(), Default::default())
    }
}

impl<I: Input, S: StringPool, B: Buffer> core::iter::Iterator for ImportsComponent<I, S, B> {
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
impl<I: Input, S: StringPool, B: Buffer> core::iter::ExactSizeIterator
    for ImportsComponent<I, S, B>
{
    fn len(&self) -> usize {
        self.count
    }
}

impl<I: Input + Debug, S: StringPool, B: Buffer> Debug for ImportsComponent<I, S, B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TypesComponent")
            .field("count", &self.count)
            .field("parser", &self.parser)
            .finish_non_exhaustive()
    }
}
