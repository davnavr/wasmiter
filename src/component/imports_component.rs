use crate::{
    component,
    input::Input,
    parser::{self, name::Name, Result, ResultExt as _, Vector},
    types,
};
use core::fmt::{Debug, Formatter};

/// Describes what kind of entity is specified by an [`Import`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum ImportKind {
    /// An imported function with the specified signature.
    Function(crate::index::TypeIdx),
    /// An imported table with the specified limits and element type.
    Table(types::TableType),
    /// An imported table with the specified limits.
    Memory(types::MemType),
    /// An imported global with the specified type.
    Global(types::GlobalType),
    /// An imported tag, introduced as part of the
    /// [exception handling proposal](https://github.com/WebAssembly/exception-handling).
    Tag(component::Tag),
}

impl ImportKind {
    /// Returns `true` if and only if the import is a function, table, memory, or global.
    pub fn is_mvp_supported(&self) -> bool {
        matches!(
            self,
            Self::Function(_) | Self::Table(_) | Self::Memory(_) | Self::Global(_)
        )
    }
}

/// Represents a
/// [WebAssembly import](https://webassembly.github.io/spec/core/binary/modules.html#import-section).
///
/// Note that importing more than one memory requires the
/// [multi-memory proposal](https://github.com/WebAssembly/multi-memory).
#[derive(Clone, Copy)]
pub struct Import<I: Input> {
    module: Name<I>,
    name: Name<I>,
    kind: ImportKind,
}

impl<I: Input> Import<I> {
    /// Gets the name of the module that this import originates from.
    #[inline]
    pub fn module(&self) -> &Name<I> {
        &self.module
    }

    /// Gets the name of the import.
    #[inline]
    pub fn name(&self) -> &Name<I> {
        &self.name
    }

    /// Gets the kind of import.
    #[inline]
    pub fn kind(&self) -> &ImportKind {
        &self.kind
    }
}

impl<'a, I: Input> Import<&'a I> {
    fn parse(offset: &mut u64, input: &'a I) -> Result<Self> {
        let module = parser::name::parse(offset, input).context("module name")?;
        let name = parser::name::parse(offset, input).context("import name")?;

        let kind_offset = *offset;
        let kind = match parser::one_byte_exact(offset, input).context("import kind")? {
            0 => ImportKind::Function(
                component::index(offset, input).context("function import type")?,
            ),
            1 => ImportKind::Table(
                component::table_type(offset, input).context("table import type")?,
            ),
            2 => ImportKind::Memory(
                component::mem_type(offset, input).context("memory import type")?,
            ),
            3 => ImportKind::Global(
                component::global_type(offset, input).context("global import type")?,
            ),
            4 => ImportKind::Tag(component::tag(offset, input).context("tag import")?),
            bad => {
                #[inline(never)]
                #[cold]
                fn bad_kind(offset: u64, kind: u8) -> parser::Error {
                    parser::Error::new(parser::ErrorKind::BadImportKind(kind))
                        .with_location_context("import section entry", offset)
                }

                return Err(bad_kind(kind_offset, bad));
            }
        };

        Ok(Self { module, name, kind })
    }
}

impl<I: Clone + Input> Import<&I> {
    /// Clones the [`Import`] by cloning the underlying [`Bytes`].
    pub fn cloned(&self) -> Import<I> {
        Import {
            module: self.module.really_cloned(),
            name: self.name.really_cloned(),
            kind: self.kind,
        }
    }
}

impl<I: Input> Debug for Import<I> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Import")
            .field("module", &self.module)
            .field("name", &self.name)
            .field("kind", &self.kind)
            .finish()
    }
}

/// Represents the
/// [**imports** component](https://webassembly.github.io/spec/core/syntax/modules.html#imports) of
/// a WebAssembly module, stored in and parsed from the
/// [*import section*](https://webassembly.github.io/spec/core/binary/modules.html#import-section).
#[derive(Clone, Copy)]
pub struct ImportsComponent<I: Input> {
    imports: Vector<u64, I>,
}

impl<I: Input> From<Vector<u64, I>> for ImportsComponent<I> {
    #[inline]
    fn from(imports: Vector<u64, I>) -> Self {
        Self { imports }
    }
}

impl<I: Input> ImportsComponent<I> {
    /// Uses the given [`Input`] to read the contents of the *import section* of a
    /// module, starting at the given `offset`.
    pub fn new(offset: u64, input: I) -> Result<Self> {
        Vector::parse(offset, input)
            .context("at start of import section")
            .map(Self::from)
    }

    /// Gets the expected remaining number of imports that have yet to be parsed.
    #[inline]
    pub fn remaining_count(&self) -> u32 {
        self.imports.remaining_count()
    }

    /// Parses the next import in the section.
    pub fn parse(&mut self) -> Result<Option<Import<&I>>> {
        self.imports
            .advance(Import::parse)
            .transpose()
            .context("within import section")
    }

    pub(crate) fn borrowed(&self) -> ImportsComponent<&I> {
        self.imports.borrowed().into()
    }
}

impl<I: Clone + Input> Iterator for ImportsComponent<I> {
    type Item = Result<Import<I>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.parse() {
            Ok(None) => None,
            Err(e) => Some(Err(e)),
            Ok(Some(import)) => Some(Ok(import.cloned())),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.imports.size_hint()
    }
}

impl<I: Clone + Input> core::iter::FusedIterator for ImportsComponent<I> {}

impl<I: Input> Debug for ImportsComponent<I> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.borrowed()).finish()
    }
}
