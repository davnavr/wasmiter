use crate::buffer::Buffer;
use crate::bytes::Bytes;
use crate::component;
use crate::index;
use crate::parser::{self, Result, ResultExt};

/// Describes what kind of entity is specified by an [`Export`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[allow(missing_docs)]
pub enum ExportKind {
    Function(index::FuncIdx),
    Table(index::TableIdx),
    Memory(index::MemIdx),
    Global(index::GlobalIdx),
}

/// Represents a
/// [WebAssembly export](https://webassembly.github.io/spec/core/binary/modules.html#export-section).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Export<'a> {
    name: &'a str,
    kind: ExportKind,
}

impl<'a> Export<'a> {
    /// Gets the name of the export.
    #[inline]
    pub fn name(&self) -> &'a str {
        self.name
    }

    /// Gets the kind of export.
    #[inline]
    pub fn kind(&self) -> &ExportKind {
        &self.kind
    }

    #[inline]
    fn parse<'b: 'a, B: Bytes, U: Buffer>(
        offset: &mut u64,
        bytes: &B,
        buffer: &'b mut U,
    ) -> Result<Self> {
        let name = parser::name(offset, bytes, buffer).context("export name")?;

        let kind = match parser::one_byte_exact(offset, bytes).context("export kind")? {
            0 => ExportKind::Function(component::index(offset, bytes).context("function export")?),
            1 => ExportKind::Table(component::index(offset, bytes).context("table export")?),
            2 => ExportKind::Memory(component::index(offset, bytes).context("memory export")?),
            3 => ExportKind::Global(component::index(offset, bytes).context("global export")?),
            bad => {
                return Err(crate::parser_bad_format!(
                    "{bad:#02X} is not a known export kind"
                ))
            }
        };

        Ok(Self { name, kind })
    }
}

/// Represents the
/// [**exports** component](https://webassembly.github.io/spec/core/syntax/modules.html#exports) of
/// a WebAssembly module, stored in and parsed from the
/// [*export section*](https://webassembly.github.io/spec/core/binary/modules.html#export-section).
#[derive(Clone, Copy)]
pub struct ExportsComponent<B: Bytes> {
    count: u32,
    offset: u64,
    bytes: B,
}

impl<B: Bytes> ExportsComponent<B> {
    /// Uses the given [`Bytes`] to read the contents of the *export section* of a module, starting
    /// at the given `offset`.
    pub fn new(mut offset: u64, bytes: B) -> Result<Self> {
        Ok(Self {
            count: parser::leb128::u32(&mut offset, &bytes).context("export count")?,
            offset,
            bytes,
        })
    }

    /// Parses the next export in the section.
    pub fn parse_with_buffer<'n, N: Buffer>(
        &mut self,
        name_buffer: &'n mut N,
    ) -> Result<Option<Export<'n>>> {
        if self.count == 0 {
            return Ok(None);
        }

        match Export::parse(&mut self.offset, &self.bytes, name_buffer) {
            Ok(export) => {
                self.count -= 1;
                Ok(Some(export))
            }
            Err(e) => {
                self.count = 0;
                Err(e)
            }
        }
    }

    fn borrowed(&self) -> ExportsComponent<&B> {
        ExportsComponent {
            count: self.count,
            offset: self.offset,
            bytes: &self.bytes,
        }
    }

    /// Gets the expected remaining number of entires in the *export section* that have yet to be parsed.
    #[inline]
    pub fn len(&self) -> u32 {
        self.count
    }

    /// Returns a value indicating if the *export section* is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<B: Bytes> core::fmt::Debug for ExportsComponent<B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // This is COMPLETELY duplicated from crate::sections::SectionSequence
        #[cfg(feature = "alloc")]
        return {
            let mut buffer = smallvec::smallvec_inline![0u8; 64];
            let mut list = f.debug_list();

            let mut sequence = self.borrowed();
            while let Some(section) = sequence.parse_with_buffer(&mut buffer).transpose() {
                list.entry(&section);
            }

            list.finish()
        };

        #[cfg(not(feature = "alloc"))]
        return f
            .debug_struct("ExportsComponent")
            .field("count", &self.count)
            .field("offset", &self.offset)
            .field("bytes", &crate::bytes::DebugBytes::from(&self.bytes))
            .finish();
    }
}
