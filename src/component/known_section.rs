use crate::allocator::Allocator;
use crate::bytes::{Bytes, Window};
use crate::component;
use crate::parser::{self, ResultExt as _};
use crate::sections::id as section_id;
use crate::{Section, SectionKind};

/// Represents a well-known WebAssembly [`Section`].
pub enum KnownSection<B: Bytes, A: Allocator> {
    /// The
    /// [*type section*](https://webassembly.github.io/spec/core/binary/modules.html#type-section).
    Type(component::TypesComponent<B>),
    /// The
    /// [*import section*](https://webassembly.github.io/spec/core/binary/modules.html#import-section).
    Import(component::ImportsComponent<B, A>),
    /// The
    /// [*function section*](https://webassembly.github.io/spec/core/binary/modules.html#function-section)
    Function(component::FunctionSection<B>),
    /// The
    /// [*table section*](https://webassembly.github.io/spec/core/binary/modules.html#table-section).
    Table(component::TablesComponent<B>),
    /// The
    /// [*memory section*](https://webassembly.github.io/spec/core/binary/modules.html#memory-section).
    Memory(component::MemsComponent<B>),
    /// The
    /// [*global section*](https://webassembly.github.io/spec/core/binary/modules.html#global-section)
    Global(component::GlobalsComponent<B>),
    /// The
    /// [*export section*](https://webassembly.github.io/spec/core/binary/modules.html#export-section)
    Export(component::ExportsComponent<B, A>),
    /// Represents the
    /// [**start** component](https://webassembly.github.io/spec/core/syntax/modules.html#start-function)
    /// of a WebAssembly module, encoded in the
    /// [*start section*](https://webassembly.github.io/spec/core/binary/modules.html#start-section).
    Start(component::FuncIdx),
    /// The
    /// [*element section*](https://webassembly.github.io/spec/core/binary/modules.html#element-section)
    Element(component::ElemsComponent<B>),
    /// The
    /// [*code section*](https://webassembly.github.io/spec/core/binary/modules.html#code-section)
    Code(component::CodeSection<B>),
    /// The
    /// [*data section*](https://webassembly.github.io/spec/core/binary/modules.html#data-section)
    Data(component::DatasComponent<B>),
}

impl<B: Bytes, A: Allocator> KnownSection<Window<B>, A> {
    /// Attempts to interpret the contents of the given WebAssembly [`Section`].
    pub fn try_from_with_allocator<S: AsRef<str>>(
        section: Section<B, S>,
        allocator: A,
    ) -> Result<parser::Result<Self>, Section<B, S>> {
        if let SectionKind::Id(id) = section.kind() {
            Ok(match *id {
                section_id::TYPE => {
                    let contents = section.into_contents();
                    component::TypesComponent::new(contents.base(), contents).map(Self::from)
                }
                section_id::IMPORT => {
                    let contents = section.into_contents();
                    component::ImportsComponent::with_allocator(
                        contents.base(),
                        contents,
                        allocator,
                    )
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
                section_id::START => {
                    let contents = section.into_contents();
                    component::index(&mut contents.base(), contents)
                        .context("start section")
                        .map(|index| Self::Start(index))
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
                _ => return Err(section),
            })
        } else {
            Err(section)
        }
    }
}

impl<B: Bytes, A: Allocator> From<component::TypesComponent<B>> for KnownSection<B, A> {
    #[inline]
    fn from(types: component::TypesComponent<B>) -> Self {
        Self::Type(types)
    }
}

impl<B: Bytes, A: Allocator> From<component::ImportsComponent<B, A>> for KnownSection<B, A> {
    #[inline]
    fn from(imports: component::ImportsComponent<B, A>) -> Self {
        Self::Import(imports)
    }
}

impl<B: Bytes, A: Allocator> From<component::FunctionSection<B>> for KnownSection<B, A> {
    #[inline]
    fn from(functions: component::FunctionSection<B>) -> Self {
        Self::Function(functions)
    }
}

impl<B: Bytes, A: Allocator> From<component::TablesComponent<B>> for KnownSection<B, A> {
    #[inline]
    fn from(tables: component::TablesComponent<B>) -> Self {
        Self::Table(tables)
    }
}

impl<B: Bytes, A: Allocator> From<component::MemsComponent<B>> for KnownSection<B, A> {
    #[inline]
    fn from(memories: component::MemsComponent<B>) -> Self {
        Self::Memory(memories)
    }
}

impl<B: Bytes, A: Allocator> From<component::GlobalsComponent<B>> for KnownSection<B, A> {
    #[inline]
    fn from(globals: component::GlobalsComponent<B>) -> Self {
        Self::Global(globals)
    }
}

impl<B: Bytes, A: Allocator> From<component::ExportsComponent<B, A>> for KnownSection<B, A> {
    #[inline]
    fn from(exports: component::ExportsComponent<B, A>) -> Self {
        Self::Export(exports)
    }
}

impl<B: Bytes, A: Allocator> From<component::ElemsComponent<B>> for KnownSection<B, A> {
    #[inline]
    fn from(elements: component::ElemsComponent<B>) -> Self {
        Self::Element(elements)
    }
}

impl<B: Bytes, A: Allocator> From<component::CodeSection<B>> for KnownSection<B, A> {
    #[inline]
    fn from(code: component::CodeSection<B>) -> Self {
        Self::Code(code)
    }
}

impl<B: Bytes, A: Allocator> From<component::DatasComponent<B>> for KnownSection<B, A> {
    #[inline]
    fn from(data: component::DatasComponent<B>) -> Self {
        Self::Data(data)
    }
}

impl<B: Bytes, A: Allocator> core::fmt::Debug for KnownSection<B, A> {
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
        }
    }
}

impl<B, A> Clone for KnownSection<B, A>
where
    B: Bytes + Clone,
    A: Allocator + Clone,
    A::Buf: Clone,
{
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
        }
    }
}

impl<B, A> Copy for KnownSection<B, A>
where
    B: Bytes + Copy,
    A: Allocator + Copy,
    A::Buf: Copy,
{
}
