use crate::allocator::{Allocator, StringPool};
use crate::component;
use crate::parser::{self, input::Input};
use crate::{section_id, Section, SectionKind};

/// Represents a well-known WebAssembly [`Section`].
#[derive(Debug)]
pub enum KnownSection<I: Input, A: Allocator, S: StringPool> {
    /// The
    /// [*type section*](https://webassembly.github.io/spec/core/binary/modules.html#type-section).
    Type(component::TypesComponent<I, A>),
    /// The
    /// [*import section*](https://webassembly.github.io/spec/core/binary/modules.html#import-section).
    Import(component::ImportsComponent<I, S, A::Buf>),
    /// The
    /// [*function section*](https://webassembly.github.io/spec/core/binary/modules.html#function-section)
    Function(component::FunctionSection<I>),
    /// The
    /// [*table section*](https://webassembly.github.io/spec/core/binary/modules.html#table-section).
    Table(component::TablesComponent<I>),
    /// The
    /// [*memory section*](https://webassembly.github.io/spec/core/binary/modules.html#memory-section).
    Memory(component::MemsComponent<I>),
}

impl<I: Input, A: Allocator, S: StringPool> KnownSection<I, A, S> {
    /// Attempts to interpret the contents of the given WebAssembly [`Section`].
    pub fn with_allocator<N: AsRef<str>>(
        section: Section<I, N>,
        allocator: A,
        string_pool: S,
    ) -> Result<parser::Result<Self>, Section<I, N>> {
        if let SectionKind::Id(id) = section.kind() {
            Ok(match *id {
                section_id::TYPE => {
                    component::TypesComponent::with_allocator(section.into_contents(), allocator)
                        .map(Self::from)
                }
                section_id::IMPORT => component::ImportsComponent::with_string_cache_and_buffer(
                    section.into_contents(),
                    allocator.allocate_buffer(),
                    string_pool,
                )
                .map(Self::from),
                section_id::FUNC => {
                    component::FunctionSection::new(section.into_contents()).map(Self::from)
                }
                section_id::TABLE => {
                    component::TablesComponent::new(section.into_contents()).map(Self::from)
                }
                section_id::MEMORY => {
                    component::MemsComponent::new(section.into_contents()).map(Self::from)
                }
                _ => return Err(section),
            })
        } else {
            Err(section)
        }
    }
}

impl<I: Input, A: Allocator, S: StringPool> From<component::TypesComponent<I, A>>
    for KnownSection<I, A, S>
{
    #[inline]
    fn from(types: component::TypesComponent<I, A>) -> Self {
        Self::Type(types)
    }
}

impl<I: Input, A: Allocator, S: StringPool> From<component::ImportsComponent<I, S, A::Buf>>
    for KnownSection<I, A, S>
{
    #[inline]
    fn from(imports: component::ImportsComponent<I, S, A::Buf>) -> Self {
        Self::Import(imports)
    }
}

impl<I: Input, A: Allocator, S: StringPool> From<component::FunctionSection<I>>
    for KnownSection<I, A, S>
{
    #[inline]
    fn from(functions: component::FunctionSection<I>) -> Self {
        Self::Function(functions)
    }
}

impl<I: Input, A: Allocator, S: StringPool> From<component::TablesComponent<I>>
    for KnownSection<I, A, S>
{
    #[inline]
    fn from(tables: component::TablesComponent<I>) -> Self {
        Self::Table(tables)
    }
}

impl<I: Input, A: Allocator, S: StringPool> From<component::MemsComponent<I>>
    for KnownSection<I, A, S>
{
    #[inline]
    fn from(memories: component::MemsComponent<I>) -> Self {
        Self::Memory(memories)
    }
}
