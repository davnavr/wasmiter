use crate::{
    bytes::Bytes,
    component,
    parser::{self, name::Name, Result, ResultExt},
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
pub struct Import<B: Bytes> {
    module: Name<B>,
    name: Name<B>,
    kind: ImportKind,
}

impl<B: Bytes> Import<B> {
    /// Gets the name of the module that this import originates from.
    #[inline]
    pub fn module(&self) -> &Name<B> {
        &self.module
    }

    /// Gets the name of the import.
    #[inline]
    pub fn name(&self) -> &Name<B> {
        &self.name
    }

    /// Gets the kind of import.
    #[inline]
    pub fn kind(&self) -> &ImportKind {
        &self.kind
    }
}

impl<'a, B: Bytes> Import<&'a B> {
    #[inline]
    fn parse(offset: &mut u64, bytes: &'a B) -> Result<Self> {
        let module = parser::name::parse(offset, bytes).context("module name")?;
        let name = parser::name::parse(offset, bytes).context("import name")?;

        let kind_offset = *offset;
        let kind = match parser::one_byte_exact(offset, bytes).context("import kind")? {
            0 => ImportKind::Function(
                component::index(offset, bytes).context("function import type")?,
            ),
            1 => ImportKind::Table(
                component::table_type(offset, bytes).context("table import type")?,
            ),
            2 => ImportKind::Memory(
                component::mem_type(offset, bytes).context("memory import type")?,
            ),
            3 => ImportKind::Global(
                component::global_type(offset, bytes).context("global import type")?,
            ),
            4 => ImportKind::Tag(component::tag(offset, bytes).context("tag import")?),
            bad => {
                return Err(crate::parser_bad_format_at_offset!(
                    "input" @ kind_offset,
                    "{bad:#04X} is not a known import kind"
                ))
            }
        };

        Ok(Self { module, name, kind })
    }
}

impl<B: Clone + Bytes> Import<&B> {
    /// Clones the [`Import`] by cloning the underlying [`Bytes`].
    pub fn cloned(&self) -> Import<B> {
        Import {
            module: self.module.really_cloned(),
            name: self.name.really_cloned(),
            kind: self.kind,
        }
    }
}

impl<B: Bytes> Debug for Import<B> {
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
/// [*imports section*](https://webassembly.github.io/spec/core/binary/modules.html#import-section).
#[derive(Clone, Copy)]
pub struct ImportsComponent<B: Bytes> {
    count: u32,
    offset: u64,
    bytes: B,
}

impl<B: Bytes> ImportsComponent<B> {
    /// Uses the given [`Bytes`] to read the contents of the *imports section* of a
    /// module, starting at the given `offset`.
    pub fn new(mut offset: u64, bytes: B) -> Result<Self> {
        Ok(Self {
            count: parser::leb128::u32(&mut offset, &bytes).context("import count")?,
            offset,
            bytes,
        })
    }

    /// Gets the expected remaining number of imports that have yet to be parsed.
    #[inline]
    pub fn len(&self) -> u32 {
        self.count
    }

    /// Returns a value indicating if the *imports section* is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Parses the next import in the section.
    pub fn parse(&mut self) -> Result<Option<Import<&B>>> {
        if self.count == 0 {
            return Ok(None);
        }

        let result = Import::parse(&mut self.offset, &self.bytes);

        if result.is_ok() {
            self.count -= 1;
        } else {
            self.count = 0;
        }

        result.map(Some)
    }

    pub(crate) fn borrowed(&self) -> ImportsComponent<&B> {
        ImportsComponent {
            count: self.count,
            offset: self.offset,
            bytes: &self.bytes,
        }
    }
}

impl<B: Clone + Bytes> Iterator for ImportsComponent<B> {
    type Item = Result<Import<B>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.parse()
            .map(|result| result.map(|i| i.cloned()))
            .transpose()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        ((self.count > 0).into(), self.count.try_into().ok())
    }
}

impl<B: Clone + Bytes> core::iter::FusedIterator for ImportsComponent<B> {}

impl<B: Bytes> Debug for ImportsComponent<B> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.borrowed()).finish()
    }
}
