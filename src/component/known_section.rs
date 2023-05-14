use crate::allocator::{self, Allocator};
use crate::bytes::{Bytes, Window};
use crate::component;
use crate::parser;
use crate::sections::id as section_id;
use crate::{Section, SectionKind};

/// Represents a well-known WebAssembly [`Section`].
#[derive(Debug)]
pub enum KnownSection<B: Bytes, A: Allocator> {
    /// The
    /// [*type section*](https://webassembly.github.io/spec/core/binary/modules.html#type-section).
    Type(component::TypesComponent<B>),
    /// The
    /// [*import section*](https://webassembly.github.io/spec/core/binary/modules.html#import-section).
    Import(component::ImportsComponent<B, A::Buf>),
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
}

impl<B: Bytes, A: Allocator> KnownSection<Window<B>, A> {
    /// Attempts to interpret the contents of the given WebAssembly [`Section`].
    pub fn try_from_with_allocator<S: AsRef<str>>(
        section: Section<B, S>,
        allocator: A,
    ) -> Result<parser::Result<Self>, Section<B, S>> {
        if let SectionKind::Id(id) = section.kind() {
            let contents = section.into_contents();
            Ok(match *id {
                section_id::TYPE => {
                    component::TypesComponent::new(contents.base(), contents).map(Self::from)
                }
                section_id::IMPORT => component::ImportsComponent::with_buffer(
                    contents.base(),
                    contents,
                    allocator.allocate_buffer(),
                )
                .map(Self::from),
                section_id::FUNC => {
                    component::FunctionSection::new(contents.base(), contents).map(Self::from)
                }
                section_id::TABLE => {
                    component::TablesComponent::new(contents.base(), contents).map(Self::from)
                }
                section_id::MEMORY => {
                    component::MemsComponent::new(contents.base(), contents).map(Self::from)
                }
                section_id::GLOBAL => {
                    component::GlobalsComponent::new(contents.base(), contents).map(Self::from)
                }
                _ => return Err(section),
            })
        } else {
            Err(section)
        }
    }
}

impl<B: Bytes, S: AsRef<str>> TryFrom<Section<B, S>>
    for KnownSection<Window<B>, allocator::Global>
{
    type Error = Section<B, S>;

    #[inline]
    fn try_from(section: Section<B, S>) -> parser::Result<Self> {
        KnownSection::try_from_with_allocator(section, allocator::Global)
    }
}

impl<B: Bytes, A: Allocator> From<component::TypesComponent<B>> for KnownSection<B, A> {
    #[inline]
    fn from(types: component::TypesComponent<B>) -> Self {
        Self::Type(types)
    }
}

impl<B: Bytes, A: Allocator> From<component::ImportsComponent<B, A::Buf>> for KnownSection<B, A> {
    #[inline]
    fn from(imports: component::ImportsComponent<B, A::Buf>) -> Self {
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
