use crate::allocator::{Allocator, StringPool};
use crate::component;
use crate::parser;
use crate::parser::input::{Input, Window};
use crate::{section_id, Section, SectionKind};

/// Represents a well-known WebAssembly [`Section`].
#[derive(Debug)]
pub enum KnownSection<I, A, S>
where
    I: Input,
    A: Allocator,
    S: StringPool<Interned = A::String>,
{
    /// The
    /// [*type section*](https://webassembly.github.io/spec/core/binary/modules.html#type-section).
    Type(component::TypesComponent<I>),
    /// The
    /// [*import section*](https://webassembly.github.io/spec/core/binary/modules.html#import-section).
    Import(component::ImportsComponent<I, A, S>),
    /// The
    /// [*function section*](https://webassembly.github.io/spec/core/binary/modules.html#function-section)
    Function(component::FunctionSection<I>),
    /// The
    /// [*table section*](https://webassembly.github.io/spec/core/binary/modules.html#table-section).
    Table(component::TablesComponent<I>),
    /// The
    /// [*memory section*](https://webassembly.github.io/spec/core/binary/modules.html#memory-section).
    Memory(component::MemsComponent<I>),
    /// The
    /// [*global section*](https://webassembly.github.io/spec/core/binary/modules.html#global-section)
    Global(component::GlobalsComponent<I>),
}

impl<I, A, S> KnownSection<Window<I>, A, S>
where
    I: Input,
    A: Allocator,
    S: StringPool<Interned = A::String>,
{
    /// Attempts to interpret the contents of the given WebAssembly [`Section`].
    pub fn with_allocator<N: AsRef<str>>(
        section: Section<I, N>,
        allocator: A,
        string_pool: S,
    ) -> Result<parser::Result<Self>, Section<I, N>> {
        if let SectionKind::Id(id) = section.kind() {
            Ok(match *id {
                section_id::TYPE => {
                    component::TypesComponent::new(section.into_contents()).map(Self::from)
                }
                section_id::IMPORT => component::ImportsComponent::with_allocator_and_string_pool(
                    section.into_contents(),
                    allocator,
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
                section_id::GLOBAL => {
                    component::GlobalsComponent::new(section.into_contents()).map(Self::from)
                }
                _ => return Err(section),
            })
        } else {
            Err(section)
        }
    }
}

impl<I, A, S> From<component::TypesComponent<I>> for KnownSection<I, A, S>
where
    I: Input,
    A: Allocator,
    S: StringPool<Interned = A::String>,
{
    #[inline]
    fn from(types: component::TypesComponent<I>) -> Self {
        Self::Type(types)
    }
}

impl<I, A, S> From<component::ImportsComponent<I, A, S>> for KnownSection<I, A, S>
where
    I: Input,
    A: Allocator,
    S: StringPool<Interned = A::String>,
{
    #[inline]
    fn from(imports: component::ImportsComponent<I, A, S>) -> Self {
        Self::Import(imports)
    }
}

impl<I, A, S> From<component::FunctionSection<I>> for KnownSection<I, A, S>
where
    I: Input,
    A: Allocator,
    S: StringPool<Interned = A::String>,
{
    #[inline]
    fn from(functions: component::FunctionSection<I>) -> Self {
        Self::Function(functions)
    }
}

impl<I, A, S> From<component::TablesComponent<I>> for KnownSection<I, A, S>
where
    I: Input,
    A: Allocator,
    S: StringPool<Interned = A::String>,
{
    #[inline]
    fn from(tables: component::TablesComponent<I>) -> Self {
        Self::Table(tables)
    }
}

impl<I, A, S> From<component::MemsComponent<I>> for KnownSection<I, A, S>
where
    I: Input,
    A: Allocator,
    S: StringPool<Interned = A::String>,
{
    #[inline]
    fn from(memories: component::MemsComponent<I>) -> Self {
        Self::Memory(memories)
    }
}

impl<I, A, S> From<component::GlobalsComponent<I>> for KnownSection<I, A, S>
where
    I: Input,
    A: Allocator,
    S: StringPool<Interned = A::String>,
{
    #[inline]
    fn from(globals: component::GlobalsComponent<I>) -> Self {
        Self::Global(globals)
    }
}
