use crate::allocator::Allocator;
use crate::component;
use crate::parser;
use crate::bytes::{Bytes, Window};
use crate::{Section, SectionKind};
use crate::sections::id as section_id;

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
    Global(component::GlobalsComponent<I>),
}

impl<B: Bytes, A: Allocator> KnownSection<Window<B>, A>
{
    /// Attempts to interpret the contents of the given WebAssembly [`Section`].
    pub fn try_from_with_allocator<N: AsRef<str>>(
        section: Section<B, N>,
        allocator: A,
    ) -> Result<parser::Result<Self>, Section<B, N>> {
        if let SectionKind::Id(id) = section.kind() {
            let contents = section.into_contents();
            Ok(match *id {
                section_id::TYPE => {
                    component::TypesComponent::new(contents.base(), contents).map(Self::from)
                }
                section_id::IMPORT => component::ImportsComponent::with_buffer(
                    contents.base(), contents,
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
                    component::GlobalsComponent::new(section.into_contents()).map(Self::from)
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

impl<B: Bytes, A: Allocator> From<component::ImportsComponent<B, A::Buf>> for KnownSection<B, A>
{
    #[inline]
    fn from(imports: component::ImportsComponent<B, A::Buf>) -> Self {
        Self::Import(imports)
    }
}

impl<B: Bytes, A: Allocator> From<component::FunctionSection<B>> for KnownSection<B, A>
{
    #[inline]
    fn from(functions: component::FunctionSection<B>) -> Self {
        Self::Function(functions)
    }
}

impl<B: Bytes, A: Allocator> From<component::TablesComponent<B>> for KnownSection<B, A>
{
    #[inline]
    fn from(tables: component::TablesComponent<B>) -> Self {
        Self::Table(tables)
    }
}

impl<B: Bytes, A: Allocator> From<component::MemsComponent<B>> for KnownSection<B, A>
{
    #[inline]
    fn from(memories: component::MemsComponent<B>) -> Self {
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
