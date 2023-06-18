use crate::{
    component, index,
    input::{BorrowInput, CloneInput, HasInput, Input},
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
pub struct Export<I: Input> {
    name: Name<I>,
    kind: ExportKind,
}

impl<I: Input> Export<I> {
    /// Gets the name of the export.
    #[inline]
    pub fn name(&self) -> &Name<I> {
        &self.name
    }

    /// Gets the kind of export.
    #[inline]
    pub fn kind(&self) -> &ExportKind {
        &self.kind
    }
}

impl<'a, I: Input> Export<&'a I> {
    fn parse(offset: &mut u64, input: &'a I) -> Result<Self> {
        let name = parser::name::parse(offset, input).context("export name")?;

        let kind_offset = *offset;
        let kind = match parser::one_byte_exact(offset, input).context("export kind")? {
            0 => ExportKind::Function(component::index(offset, input).context("function export")?),
            1 => ExportKind::Table(component::index(offset, input).context("table export")?),
            2 => ExportKind::Memory(component::index(offset, input).context("memory export")?),
            3 => ExportKind::Global(component::index(offset, input).context("global export")?),
            4 => ExportKind::Tag(component::index(offset, input).context("tag export")?),
            bad => {
                #[inline(never)]
                #[cold]
                fn bad_kind(offset: u64, kind: u8) -> parser::Error {
                    parser::Error::new(parser::ErrorKind::BadExportKind(kind))
                        .with_location_context("export section entry", offset)
                }

                return Err(bad_kind(kind_offset, bad));
            }
        };

        Ok(Self { name, kind })
    }
}

impl<I: Input> HasInput<I> for Export<I> {
    #[inline]
    fn input(&self) -> &I {
        self.name.input()
    }
}

impl<'a, I: Clone + Input + 'a> CloneInput<'a, I> for Export<&'a I> {
    type Cloned = Export<I>;

    #[inline]
    fn clone_input(&self) -> Export<I> {
        Export {
            name: self.name.clone_input(),
            kind: self.kind,
        }
    }
}

impl<I: Input> Debug for Export<I> {
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
pub struct ExportsComponent<I: Input> {
    exports: Vector<u64, I>,
}

impl<I: Input> From<Vector<u64, I>> for ExportsComponent<I> {
    #[inline]
    fn from(exports: Vector<u64, I>) -> Self {
        Self { exports }
    }
}

impl<I: Input> ExportsComponent<I> {
    /// Uses the given [`Input`] to read the contents of the *export section* of a module, starting
    /// at the given `offset`.
    pub fn new(offset: u64, bytes: I) -> Result<Self> {
        Vector::parse(offset, bytes)
            .context("at start of export section")
            .map(Self::from)
    }

    /// Parses the next export in the section.
    pub fn parse(&mut self) -> Result<Option<Export<&I>>> {
        self.exports
            .advance(Export::parse)
            .transpose()
            .context("within export section")
    }

    /// Gets the expected remaining number of entires in the *export section* that have yet to be parsed.
    #[inline]
    pub fn remaining_count(&self) -> u32 {
        self.exports.remaining_count()
    }
}

impl<I: Input> HasInput<I> for ExportsComponent<I> {
    #[inline]
    fn input(&self) -> &I {
        self.exports.input()
    }
}

impl<'a, I: Input + 'a> BorrowInput<'a, I> for ExportsComponent<I> {
    type Borrowed = ExportsComponent<&'a I>;

    #[inline]
    fn borrow_input(&'a self) -> Self::Borrowed {
        self.exports.borrow_input().into()
    }
}

impl<I: Clone + Input> Iterator for ExportsComponent<I> {
    type Item = Result<Export<I>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.parse() {
            Ok(None) => None,
            Err(e) => Some(Err(e)),
            Ok(Some(export)) => Some(Ok(export.clone_input())),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.exports.size_hint()
    }
}

impl<I: Clone + Input> core::iter::FusedIterator for ExportsComponent<I> {}

impl<I: Input> core::fmt::Debug for ExportsComponent<I> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.borrow_input()).finish()
    }
}
