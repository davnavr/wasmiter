use crate::buffer::Buffer;
use crate::bytes::Bytes;
use crate::component;
use crate::parser::{self, Result, ResultExt};

/// Describes what kind of entity is specified by an [`Import`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ImportKind {
    /// An imported function with the specified signature.
    Function(crate::index::TypeIdx),
    /// An imported table with the specified limits and element type.
    Table(component::TableType),
    /// An imported table with the specified limits.
    Memory(component::MemType),
    /// An imported global with the specified type.
    Global(component::GlobalType),
}

/// Represents a
/// [WebAssembly import](https://webassembly.github.io/spec/core/binary/modules.html#import-section).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Import<'a> {
    module: &'a str,
    name: &'a str,
    kind: ImportKind,
}

impl<'a> Import<'a> {
    /// Gets the name of the module that this import originates from.
    #[inline]
    pub fn module(&self) -> &'a str {
        self.module
    }

    /// Gets the name of the import.
    #[inline]
    pub fn name(&self) -> &'a str {
        self.name
    }

    /// Gets the kind of import.
    #[inline]
    pub fn kind(&self) -> &ImportKind {
        &self.kind
    }

    #[inline]
    fn parse<'b: 'a, B: Bytes, N: Buffer>(
        offset: &mut u64,
        bytes: &B,
        buffer: &'b mut N,
    ) -> Result<Self> {
        let module_name = parser::name(offset, &bytes, buffer)
            .context("module name")?
            .len();

        let import_name = parser::name(offset, bytes, buffer)
            .context("import name")?
            .len();

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
            bad => {
                return Err(crate::parser_bad_format!(
                    "{bad:#02X} is not a known import kind"
                ))
            }
        };

        // Need to convert &mut [u8] to [u8], borrow error occurs if as_ref is used
        let names: &'a [u8] = buffer.as_mut();
        Ok(Self {
            module: {
                // Safety: parser::name returns a valid string
                unsafe { core::str::from_utf8_unchecked(&names[0..module_name]) }
            },
            name: {
                // Safety: parser::name returns a valid string, and this is after the module name
                unsafe {
                    core::str::from_utf8_unchecked(&names[module_name..module_name + import_name])
                }
            },
            kind,
        })
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
    pub fn parse_with_buffer<'n, N: Buffer>(
        &mut self,
        name_buffer: &'n mut N,
    ) -> Result<Option<Import<'n>>> {
        if self.count == 0 {
            return Ok(None);
        }

        name_buffer.clear();
        match Import::parse(&mut self.offset, &self.bytes, name_buffer) {
            Ok(import) => {
                self.count -= 1;
                Ok(Some(import))
            }
            Err(e) => {
                self.count = 0;
                Err(e)
            }
        }
    }

    fn borrowed(&self) -> ImportsComponent<&B> {
        ImportsComponent {
            count: self.count,
            offset: self.offset,
            bytes: &self.bytes,
        }
    }
}

impl<B: Bytes> core::fmt::Debug for ImportsComponent<B> {
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
            .debug_struct("ImportsComponent")
            .field("count", &self.count)
            .field("offset", &self.offset)
            .field("bytes", &crate::bytes::DebugBytes::from(&self.bytes))
            .finish();
    }
}
