use crate::allocator::{self, Allocator, OwnOrRef, StringPool};
use crate::component;
use crate::parser::{self, input::Input, Decoder, Result, ResultExt};
use core::fmt::{Debug, Formatter};

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
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Import")
            .field("module", &self.module.as_ref())
            .field("name", &self.name.as_ref())
            .field("kind", &self.kind)
            .finish()
    }
}

/// Parses an [`Import`].
#[derive(Default)]
pub struct ImportParser<A: Allocator, S: StringPool<Interned = A::String>> {
    strings: S,
    buffer: A::Buf,
    allocator: A,
}

impl<A: Allocator, S: StringPool<Interned = A::String>> ImportParser<A, S> {
    /// Creates a new [`ImportParser`].
    pub fn new(allocator: A, strings: S) -> Self {
        Self {
            strings,
            buffer: allocator.allocate_buffer(),
            allocator,
        }
    }
}

impl<A: Allocator, S: StringPool<Interned = A::String>> parser::Parse for ImportParser<A, S> {
    type Output = Import<S::Interned>;

    fn parse<I: Input>(&mut self, input: &mut Decoder<I>) -> Result<Self::Output> {
        let module_name = input.name(&mut self.buffer).context("module name")?;

        let module = if let Some(cached) = cached_module_name(module_name) {
            OwnOrRef::Reference(cached)
        } else {
            OwnOrRef::Owned(self.strings.get(module_name))
        };

        let name = self
            .allocator
            .allocate_string(input.name(&mut self.buffer).context("import name")?);

        let kind = match input.one_byte_exact().context("import kind")? {
            0 => ImportKind::Function(input.index().context("function import type")?),
            1 => ImportKind::Table(input.table_type().context("table import type")?),
            2 => ImportKind::Memory(input.mem_type().context("memory import type")?),
            3 => ImportKind::Global(input.global_type().context("global import type")?),
            bad => {
                return Err(crate::parser_bad_format!(
                    "{bad:#02X} is not a known import kind"
                ))
            }
        };

        Ok(Import { module, name, kind })
    }
}

impl<A: Allocator, S: StringPool<Interned = A::String>> Debug for ImportParser<A, S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ImportParser").finish_non_exhaustive()
    }
}

/// Represents the
/// [**imports** component](https://webassembly.github.io/spec/core/syntax/modules.html#imports) of
/// a WebAssembly module, stored in and parsed from the
/// [*imports section*](https://webassembly.github.io/spec/core/binary/modules.html#import-section).
pub struct ImportsComponent<I, A, S>
where
    I: Input,
    S: StringPool<Interned = A::String>,
    A: Allocator,
{
    imports: parser::Vector<I, ImportParser<A, S>>,
}

impl<I, A, S> From<parser::Vector<I, ImportParser<A, S>>> for ImportsComponent<I, A, S>
where
    I: Input,
    A: Allocator,
    S: StringPool<Interned = A::String>,
{
    #[inline]
    fn from(imports: parser::Vector<I, ImportParser<A, S>>) -> Self {
        Self { imports }
    }
}

impl<I, A, S> ImportsComponent<I, A, S>
where
    I: Input,
    A: Allocator,
    S: StringPool<Interned = A::String>,
{
    /// Uses the given [`Decoder<I>`] and [`ImportParser<A, S>`] to read the contents of the
    /// *imports section* of a module.
    pub fn with_parser(input: Decoder<I>, parser: ImportParser<A, S>) -> Result<Self> {
        parser::Vector::new(input, parser).map(Self::from)
    }

    /// Uses the given [`Decoder<I>`] to read the contents of the *imports section* of a module,
    /// using the given [`StringPool`] to intern module names and [`Allocator`] to allocate buffers
    /// for import names.
    pub fn with_allocator_and_string_pool(
        input: Decoder<I>,
        allocator: A,
        strings: S,
    ) -> Result<Self> {
        Self::with_parser(input, ImportParser::new(allocator, strings))
    }
}

#[cfg(feature = "alloc")]
impl<I: Input> ImportsComponent<I, allocator::Global, allocator::FakeStringPool> {
    /// Uses a [`Decoder<I>`] to read the contents of the *imports section* of a module.
    pub fn new(input: Decoder<I>) -> Result<Self> {
        Self::with_parser(input, Default::default())
    }
}

impl<I, A, S> core::iter::Iterator for ImportsComponent<I, A, S>
where
    I: Input,
    A: Allocator,
    S: StringPool<Interned = A::String>,
{
    type Item = Result<Import<A::String>>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.imports.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.imports.size_hint()
    }
}

impl<I, A, S> Debug for ImportsComponent<I, A, S>
where
    I: Input,
    A: Allocator,
    S: StringPool<Interned = A::String>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TypesComponent")
            .field("count", &self.imports.len())
            .finish_non_exhaustive()
    }
}
