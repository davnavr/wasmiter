use crate::component;
use crate::input::{Input, Window};
use crate::parser::{self, ResultExt as _};
use crate::sections::{id as section_id, Section};

/// Represents a well-known WebAssembly [`Section`].
#[non_exhaustive]
pub enum KnownSection<I: Input> {
    /// The
    /// [*type section*](https://webassembly.github.io/spec/core/binary/modules.html#type-section).
    Type(component::TypesComponent<I>),
    /// The
    /// [*import section*](https://webassembly.github.io/spec/core/binary/modules.html#import-section).
    Import(component::ImportsComponent<I>),
    /// The
    /// [*function section*](https://webassembly.github.io/spec/core/binary/modules.html#function-section).
    Function(component::FunctionSection<I>),
    /// The
    /// [*table section*](https://webassembly.github.io/spec/core/binary/modules.html#table-section).
    Table(component::TablesComponent<I>),
    /// The
    /// [*memory section*](https://webassembly.github.io/spec/core/binary/modules.html#memory-section).
    Memory(component::MemsComponent<I>),
    /// The
    /// [*global section*](https://webassembly.github.io/spec/core/binary/modules.html#global-section).
    Global(component::GlobalsComponent<I>),
    /// The
    /// [*export section*](https://webassembly.github.io/spec/core/binary/modules.html#export-section).
    Export(component::ExportsComponent<I>),
    /// Represents the
    /// [**start** component](https://webassembly.github.io/spec/core/syntax/modules.html#start-function)
    /// of a WebAssembly module, encoded in the
    /// [*start section*](https://webassembly.github.io/spec/core/binary/modules.html#start-section).
    Start(crate::index::FuncIdx),
    /// The
    /// [*element section*](https://webassembly.github.io/spec/core/binary/modules.html#element-section).
    Element(component::ElemsComponent<I>),
    /// The
    /// [*code section*](https://webassembly.github.io/spec/core/binary/modules.html#code-section).
    Code(component::CodeSection<I>),
    /// The
    /// [*data section*](https://webassembly.github.io/spec/core/binary/modules.html#data-section).
    Data(component::DatasComponent<I>),
    /// The
    /// [*data count section*](https://webassembly.github.io/spec/core/binary/modules.html#data-count-section)
    /// specifies the number of of entries in the [*data section*](KnownSection::Data).
    DataCount(u32),
    /// The
    /// [*tag section*](https://webassembly.github.io/exception-handling/core/binary/modules.html#tag-section).
    Tag(component::TagsComponent<I>),
}

impl<I: Input> KnownSection<I> {
    /// Gets the [*id*](https://webassembly.github.io/spec/core/binary/modules.html#sections) for
    /// the section.
    pub const fn id(&self) -> u8 {
        match self {
            Self::Type(_) => section_id::TYPE,
            Self::Import(_) => section_id::IMPORT,
            Self::Function(_) => section_id::FUNC,
            Self::Table(_) => section_id::TABLE,
            Self::Memory(_) => section_id::MEMORY,
            Self::Global(_) => section_id::GLOBAL,
            Self::Export(_) => section_id::EXPORT,
            Self::Start(_) => section_id::START,
            Self::Element(_) => section_id::ELEMENT,
            Self::Code(_) => section_id::CODE,
            Self::Data(_) => section_id::DATA,
            Self::DataCount(_) => section_id::DATA_COUNT,
            Self::Tag(_) => section_id::TAG,
        }
    }

    /// Returns `true` if the section was introduced in WebAssembly 1.0 (the 2017 MVP).
    pub fn is_mvp_section(&self) -> bool {
        matches!(
            self,
            Self::Type(_)
                | Self::Import(_)
                | Self::Function(_)
                | Self::Table(_)
                | Self::Memory(_)
                | Self::Global(_)
                | Self::Export(_)
                | Self::Start(_)
                | Self::Element(_)
                | Self::Code(_)
                | Self::Data(_)
                | Self::DataCount(_)
        )
    }
}

impl<I: Input> KnownSection<Window<I>> {
    /// Attempts to interpret the contents of the given WebAssembly [`Section`].
    ///
    /// Returns `Err(_)` if the section is a custom section, or if the section's
    /// [*id*](https://webassembly.github.io/spec/core/binary/modules.html#sections) is not
    /// recognized.
    ///
    /// Returns `Ok(Err(_))` if the section **was** recognized, but an attempt to parse a length field
    /// failed.
    pub fn interpret(section: Section<I>) -> Result<parser::Parsed<Self>, Section<I>> {
        Ok(match section.id() {
            section_id::TYPE => {
                let contents = section.into_contents();
                component::TypesComponent::new(contents.base(), contents).map(Self::from)
            }
            section_id::IMPORT => {
                let contents = section.into_contents();
                component::ImportsComponent::new(contents.base(), contents)
            }
            .map(Self::from),
            section_id::FUNC => {
                let contents = section.into_contents();
                component::FunctionSection::new(contents.base(), contents).map(Self::from)
            }
            section_id::TABLE => {
                let contents = section.into_contents();
                component::TablesComponent::new(contents.base(), contents).map(Self::from)
            }
            section_id::MEMORY => {
                let contents = section.into_contents();
                component::MemsComponent::new(contents.base(), contents).map(Self::from)
            }
            section_id::GLOBAL => {
                let contents = section.into_contents();
                component::GlobalsComponent::new(contents.base(), contents).map(Self::from)
            }
            section_id::EXPORT => {
                let contents = section.into_contents();
                component::ExportsComponent::new(contents.base(), contents).map(Self::from)
            }
            section_id::START => {
                let contents = section.into_contents();
                component::index(&mut contents.base(), contents)
                    .context("start section")
                    .map(Self::Start)
            }
            section_id::ELEMENT => {
                let contents = section.into_contents();
                component::ElemsComponent::new(contents.base(), contents).map(Self::from)
            }
            section_id::CODE => {
                let contents = section.into_contents();
                component::CodeSection::new(contents.base(), contents).map(Self::from)
            }
            section_id::DATA => {
                let contents = section.into_contents();
                component::DatasComponent::new(contents.base(), contents).map(Self::from)
            }
            section_id::DATA_COUNT => {
                let contents = section.into_contents();
                parser::leb128::u32(&mut contents.base(), contents)
                    .context("data count section")
                    .map(Self::DataCount)
            }
            section_id::TAG => {
                let contents = section.into_contents();
                component::TagsComponent::new(contents.base(), contents).map(Self::from)
            }
            _ => return Err(section),
        })
    }
}

impl<I: Input> From<component::TypesComponent<I>> for KnownSection<I> {
    #[inline]
    fn from(types: component::TypesComponent<I>) -> Self {
        Self::Type(types)
    }
}

impl<I: Input> From<component::ImportsComponent<I>> for KnownSection<I> {
    #[inline]
    fn from(imports: component::ImportsComponent<I>) -> Self {
        Self::Import(imports)
    }
}

impl<I: Input> From<component::FunctionSection<I>> for KnownSection<I> {
    #[inline]
    fn from(functions: component::FunctionSection<I>) -> Self {
        Self::Function(functions)
    }
}

impl<I: Input> From<component::TablesComponent<I>> for KnownSection<I> {
    #[inline]
    fn from(tables: component::TablesComponent<I>) -> Self {
        Self::Table(tables)
    }
}

impl<I: Input> From<component::MemsComponent<I>> for KnownSection<I> {
    #[inline]
    fn from(memories: component::MemsComponent<I>) -> Self {
        Self::Memory(memories)
    }
}

impl<I: Input> From<component::GlobalsComponent<I>> for KnownSection<I> {
    #[inline]
    fn from(globals: component::GlobalsComponent<I>) -> Self {
        Self::Global(globals)
    }
}

impl<I: Input> From<component::ExportsComponent<I>> for KnownSection<I> {
    #[inline]
    fn from(exports: component::ExportsComponent<I>) -> Self {
        Self::Export(exports)
    }
}

impl<I: Input> From<component::ElemsComponent<I>> for KnownSection<I> {
    #[inline]
    fn from(elements: component::ElemsComponent<I>) -> Self {
        Self::Element(elements)
    }
}

impl<I: Input> From<component::CodeSection<I>> for KnownSection<I> {
    #[inline]
    fn from(code: component::CodeSection<I>) -> Self {
        Self::Code(code)
    }
}

impl<I: Input> From<component::DatasComponent<I>> for KnownSection<I> {
    #[inline]
    fn from(data: component::DatasComponent<I>) -> Self {
        Self::Data(data)
    }
}

impl<I: Input> From<component::TagsComponent<I>> for KnownSection<I> {
    #[inline]
    fn from(tags: component::TagsComponent<I>) -> Self {
        Self::Tag(tags)
    }
}

impl<I: Input> core::fmt::Debug for KnownSection<I> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Type(types) => f.debug_tuple("Type").field(types).finish(),
            Self::Import(imports) => f.debug_tuple("Import").field(imports).finish(),
            Self::Function(functions) => f.debug_tuple("Function").field(functions).finish(),
            Self::Table(tables) => f.debug_tuple("Table").field(tables).finish(),
            Self::Memory(memories) => f.debug_tuple("Memory").field(memories).finish(),
            Self::Global(globals) => f.debug_tuple("Global").field(globals).finish(),
            Self::Export(exports) => f.debug_tuple("Export").field(exports).finish(),
            Self::Start(start) => f.debug_tuple("Start").field(start).finish(),
            Self::Element(elements) => f.debug_tuple("Element").field(elements).finish(),
            Self::Code(code) => f.debug_tuple("Code").field(code).finish(),
            Self::Data(data) => f.debug_tuple("Data").field(data).finish(),
            Self::DataCount(count) => f.debug_tuple("DataCount").field(count).finish(),
            Self::Tag(tags) => f.debug_tuple("Tag").field(tags).finish(),
        }
    }
}

impl<I: Input + Clone> Clone for KnownSection<I> {
    fn clone(&self) -> Self {
        match self {
            Self::Type(types) => Self::Type(types.clone()),
            Self::Import(imports) => Self::Import(imports.clone()),
            Self::Function(functions) => Self::Function(functions.clone()),
            Self::Table(tables) => Self::Table(tables.clone()),
            Self::Memory(memories) => Self::Memory(memories.clone()),
            Self::Global(globals) => Self::Global(globals.clone()),
            Self::Export(exports) => Self::Export(exports.clone()),
            Self::Start(start) => Self::Start(*start),
            Self::Element(elements) => Self::Element(elements.clone()),
            Self::Code(code) => Self::Code(code.clone()),
            Self::Data(data) => Self::Data(data.clone()),
            Self::DataCount(count) => Self::DataCount(*count),
            Self::Tag(tags) => Self::Tag(tags.clone()),
        }
    }
}

impl<I: Input + Copy> Copy for KnownSection<I> {}
