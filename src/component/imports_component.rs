use crate::allocator::Buffer;
use crate::bytes::Bytes;
use crate::component;
use crate::parser::{self, Result, ResultExt};
use core::fmt::{Debug, Formatter};

/// Describes what kind of entity is specified by an [`Import`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
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
}

/// Represents the
/// [**imports** component](https://webassembly.github.io/spec/core/syntax/modules.html#imports) of
/// a WebAssembly module, stored in and parsed from the
/// [*imports section*](https://webassembly.github.io/spec/core/binary/modules.html#import-section).
pub struct ImportsComponent<B: Bytes, U: Buffer> {
    count: u32,
    offset: u64,
    bytes: B,
    buffer: U,
}

impl<B: Bytes, U: Buffer> ImportsComponent<B, U> {
    /// Uses the given [`Bytes`] and [`Buffer`] to read the contents of the *imports section* of a
    /// module, starting at the given `offset`.
    ///
    /// The `buffer` will be used to contain any parsed UTF-8 strings.
    pub fn with_buffer(mut offset: u64, bytes: B, buffer: U) -> Result<Self> {
        Ok(Self {
            count: parser::leb128::u32(&mut offset, &bytes).context("import count")?,
            offset,
            bytes,
            buffer,
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

    #[inline]
    fn parse(&mut self) -> Result<Import<'_>> {
        let module_name = 0..parser::name(&mut self.offset, &self.bytes, &mut self.buffer)
            .context("module name")?
            .len();

        let import_name: &str =
            parser::name(&mut self.offset, &self.bytes, &mut self.buffer).context("import name")?;

        let kind = match parser::one_byte_exact(&mut self.offset, &self.bytes)
            .context("import kind")?
        {
            0 => ImportKind::Function(
                component::index(&mut self.offset, &self.bytes).context("function import type")?,
            ),
            1 => ImportKind::Table(
                component::table_type(&mut self.offset, &self.bytes)
                    .context("table import type")?,
            ),
            2 => ImportKind::Memory(
                component::mem_type(&mut self.offset, &self.bytes).context("memory import type")?,
            ),
            3 => ImportKind::Global(
                component::global_type(&mut self.offset, &self.bytes)
                    .context("global import type")?,
            ),
            bad => {
                return Err(crate::parser_bad_format!(
                    "{bad:#02X} is not a known import kind"
                ))
            }
        };

        Ok(Import {
            module: unsafe {
                // Safety: parser::name returns a valid string
                core::str::from_utf8_unchecked(&self.buffer.as_ref()[module_name])
            },
            name: import_name,
            kind,
        })
    }

    /// Parses the next import in the section.
    pub fn next(&mut self) -> Result<Option<Import<'_>>> {
        if self.count == 0 {
            return Ok(None);
        }

        match self.parse() {
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
}

#[cfg(feature = "alloc")]
impl<B: Bytes> ImportsComponent<B, alloc::vec::Vec<u8>> {
    /// Uses the given [`Bytes`] to read the contents of the *imports section* of a module,
    /// starting at the given `offset`.
    pub fn new(offset: u64, bytes: B) -> Result<Self> {
        Self::with_buffer(offset, bytes, alloc::vec::Vec::new())
    }
}

impl<B: Bytes, U: Buffer> Debug for ImportsComponent<B, U> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TypesComponent")
            .field("count", &self.count)
            .finish_non_exhaustive()
    }
}
