use crate::allocator::{Allocator, StringPool};
use crate::component;
use crate::parser::{self, input::Input};
use crate::{section_id, Section, SectionKind};

/// Represents a well-known WebAssembly [`Section`].
#[derive(Debug)]
pub enum KnownSection<I: Input, A: Allocator, S: StringPool> {
    /// The
    /// [*types section*](https://webassembly.github.io/spec/core/binary/modules.html#type-section).
    Types(component::TypesComponent<I, A>),
    /// The
    /// [*imports section*](https://webassembly.github.io/spec/core/binary/modules.html#import-section).
    Imports(component::ImportsComponent<I, S, A::Buf>),
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
        Self::Types(types)
    }
}

impl<I: Input, A: Allocator, S: StringPool> From<component::ImportsComponent<I, S, A::Buf>>
    for KnownSection<I, A, S>
{
    #[inline]
    fn from(imports: component::ImportsComponent<I, S, A::Buf>) -> Self {
        Self::Imports(imports)
    }
}
