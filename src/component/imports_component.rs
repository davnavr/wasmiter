use crate::allocator::{Allocator, Buffer};
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

    #[inline]
    fn parse<'b: 'a, B: Bytes, U: Buffer>(
        offset: &mut u64,
        bytes: &B,
        buffer: &'b mut U,
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
pub struct ImportsComponent<B: Bytes, A: Allocator> {
    count: u32,
    offset: u64,
    bytes: B,
    buffer: A::Buf,
    allocator: A,
}

impl<B: Bytes, A: Allocator> ImportsComponent<B, A> {
    /// Uses the given [`Bytes`] to read the contents of the *imports section* of a
    /// module, starting at the given `offset`, using a buffer from the [`Allocator`] to contain
    /// any parsed UTF-8 strings.
    pub fn with_allocator(mut offset: u64, bytes: B, allocator: A) -> Result<Self> {
        Ok(Self {
            count: parser::leb128::u32(&mut offset, &bytes).context("import count")?,
            offset,
            bytes,
            buffer: allocator.allocate_buffer(),
            allocator,
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
    pub fn parse_next(&mut self) -> Result<Option<Import<'_>>> {
        if self.count == 0 {
            return Ok(None);
        }

        self.buffer.clear();
        match Import::parse(&mut self.offset, &self.bytes, &mut self.buffer) {
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
impl<B: Bytes> ImportsComponent<B, crate::allocator::Global> {
    /// Uses the given [`Bytes`] to read the contents of the *imports section* of a module,
    /// starting at the given `offset`.
    pub fn new(offset: u64, bytes: B) -> Result<Self> {
        Self::with_allocator(offset, bytes, Default::default())
    }
}

impl<B: Bytes, A: Allocator> Debug for ImportsComponent<B, A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut borrowed = ImportsComponent {
            count: self.count,
            offset: self.offset,
            bytes: &self.bytes,
            buffer: self.allocator.allocate_buffer(),
            allocator: &self.allocator,
        };

        let mut list = f.debug_list();
        while let Some(import) = borrowed.parse_next().transpose() {
            list.entry(&import);
        }
        list.finish()
    }
}
