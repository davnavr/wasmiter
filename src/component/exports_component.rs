use crate::{
    bytes::Bytes,
    component, index,
    parser::{self, name::Name, Result, ResultExt as _, Vector},
};
use core::fmt::{Debug, Formatter};

/// Describes what kind of entity is specified by an [`Export`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[allow(missing_docs)]
#[non_exhaustive]
pub enum ExportKind {
    Function(index::FuncIdx),
    Table(index::TableIdx),
    Memory(index::MemIdx),
    Global(index::GlobalIdx),
    /// Exports the given [`Tag`](component::Tag).
    ///
    /// Introduced as part of the
    /// [exception handling proposal](https://github.com/WebAssembly/exception-handling).
    Tag(index::TagIdx),
}

impl ExportKind {
    /// Returns `true` if and only if the import is a function, table, memory, or global.
    pub fn is_mvp_supported(&self) -> bool {
        matches!(
            self,
            Self::Function(_) | Self::Table(_) | Self::Memory(_) | Self::Global(_)
        )
    }
}

/// Represents a
/// [WebAssembly export](https://webassembly.github.io/spec/core/binary/modules.html#export-section).
#[derive(Clone, Copy)]
pub struct Export<B: Bytes> {
    name: Name<B>,
    kind: ExportKind,
}

impl<B: Bytes> Export<B> {
    /// Gets the name of the export.
    #[inline]
    pub fn name(&self) -> &Name<B> {
        &self.name
    }

    /// Gets the kind of export.
    #[inline]
    pub fn kind(&self) -> &ExportKind {
        &self.kind
    }
}

impl<'a, B: Bytes> Export<&'a B> {
    fn parse(offset: &mut u64, bytes: &'a B) -> Result<Self> {
        let name = parser::name::parse(offset, bytes).context("export name")?;

        let kind_offset = *offset;
        let kind = match parser::one_byte_exact(offset, bytes).context("export kind")? {
            0 => ExportKind::Function(component::index(offset, bytes).context("function export")?),
            1 => ExportKind::Table(component::index(offset, bytes).context("table export")?),
            2 => ExportKind::Memory(component::index(offset, bytes).context("memory export")?),
            3 => ExportKind::Global(component::index(offset, bytes).context("global export")?),
            4 => ExportKind::Tag(component::index(offset, bytes).context("tag export")?),
            bad => {
                return Err(crate::parser_bad_format_at_offset!(
                    "input" @ kind_offset,
                    "{bad:#04X} is not a known export kind"
                ))
            }
        };

        Ok(Self { name, kind })
    }
}

impl<B: Clone + Bytes> Export<&B> {
    /// Clones the [`Export`] by cloning the underlying [`Bytes`].
    pub fn cloned(&self) -> Export<B> {
        Export {
            name: self.name.really_cloned(),
            kind: self.kind,
        }
    }
}

impl<B: Bytes> Debug for Export<B> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Export")
            .field("name", &self.name)
            .field("kind", &self.kind)
            .finish()
    }
}

/// Represents the
/// [**exports** component](https://webassembly.github.io/spec/core/syntax/modules.html#exports) of
/// a WebAssembly module, stored in and parsed from the
/// [*export section*](https://webassembly.github.io/spec/core/binary/modules.html#export-section).
#[derive(Clone, Copy)]
pub struct ExportsComponent<B: Bytes> {
    exports: Vector<u64, B>,
}

impl<B: Bytes> From<Vector<u64, B>> for ExportsComponent<B> {
    #[inline]
    fn from(exports: Vector<u64, B>) -> Self {
        Self { exports }
    }
}

impl<B: Bytes> ExportsComponent<B> {
    /// Uses the given [`Bytes`] to read the contents of the *export section* of a module, starting
    /// at the given `offset`.
    pub fn new(offset: u64, bytes: B) -> Result<Self> {
        Vector::parse(offset, bytes)
            .context("at start of exports section")
            .map(Self::from)
    }

    /// Parses the next export in the section.
    pub fn parse(&mut self) -> Result<Option<Export<&B>>> {
        self.exports
            .advance(Export::parse)
            .transpose()
            .context("within exports section")
    }

    #[inline]
    pub(crate) fn borrowed(&self) -> ExportsComponent<&B> {
        self.exports.borrowed().into()
    }

    /// Gets the expected remaining number of entires in the *export section* that have yet to be parsed.
    #[inline]
    pub fn remaining_count(&self) -> u32 {
        self.exports.remaining_count()
    }
}

impl<B: Clone + Bytes> Iterator for ExportsComponent<B> {
    type Item = Result<Export<B>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.parse() {
            Ok(None) => None,
            Err(e) => Some(Err(e)),
            Ok(Some(code)) => Some(Ok(code.cloned())),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.exports.size_hint()
    }
}

impl<B: Clone + Bytes> core::iter::FusedIterator for ExportsComponent<B> {}

impl<B: Bytes> core::fmt::Debug for ExportsComponent<B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.borrowed()).finish()
    }
}
