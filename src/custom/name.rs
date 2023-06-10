//! Types to parse and describe the structure of the
//! [`name` section, described in the WebAssembly specification appendix](https://webassembly.github.io/spec/core/appendix/custom.html),
//! which associates UTF-8 [`Name`]s with definitions in a module.

use crate::{
    bytes::{Bytes, Window},
    index, parser,
    sections::{Section, SectionSequence},
};
use core::fmt::Debug;

mod name_assoc;
mod name_map;

pub use name_assoc::NameAssoc;
pub use name_map::NameMap;

/// Represents a
/// [name subsection](https://webassembly.github.io/spec/core/appendix/custom.html#subsections)
/// within the [`NameSection`].
#[derive(Clone, Copy)]
#[non_exhaustive]
pub enum NameSubsection<B: Bytes> {
    /// The
    /// [*module name subsection*](https://webassembly.github.io/spec/core/appendix/custom.html#module-names)
    /// assigns a name to the WebAssembly module.
    ModuleName(parser::name::Name<B>),
    /// The
    /// [*function name subsection*](https://webassembly.github.io/spec/core/appendix/custom.html#function-names)
    /// assigns names to the functions of a WebAssembly module.
    FunctionName(NameMap<index::FuncIdx, u64, B>),
}

impl<B: Bytes> NameSubsection<Window<B>> {
    /// Attempts to interpret the contents of the given name subsection.
    ///
    /// Returns `Err(_)` if the section's
    /// [*id*](https://webassembly.github.io/spec/core/appendix/custom.html#subsections) is not
    /// recognized.
    ///
    /// Returns `Ok(Err(_))` if the section **was** recognized, but an attempt to parse a field
    /// failed.
    pub fn interpret(section: Section<B>) -> Result<parser::Result<Self>, Section<B>> {
        match section.id() {
            0 => {
                let contents = section.into_contents();
                let mut offset = contents.base();
                Ok(parser::name::parse(&mut offset, contents).map(Self::ModuleName))
            }
            1 => {
                let contents = section.into_contents();
                Ok(NameMap::new(contents.base(), contents).map(Self::FunctionName))
            }
            _ => Err(section),
        }
    }
}

impl<B: Bytes> Debug for NameSubsection<B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ModuleName(name) => f.debug_tuple("ModuleName").field(name).finish(),
            Self::FunctionName(names) => f.debug_tuple("FunctionName").field(names).finish(),
        }
    }
}

/// Represents the sequence of subsections in the
/// [`name` section](https://webassembly.github.io/spec/core/appendix/custom.html).
#[derive(Clone, Copy)]
pub struct NameSection<B: Bytes> {
    // TODO: subsection id order
    sections: SectionSequence<B>,
}

impl<B: Bytes> NameSection<B> {
    /// Creates a [`NameSection`] from the given sequence of `sections`.
    pub fn new(sections: SectionSequence<B>) -> Self {
        Self { sections }
    }

    /// Consumes the [`NameSection`], returning the remaining subsections.
    pub fn into_sections(self) -> SectionSequence<B> {
        self.sections
    }
}

impl<B: Bytes> Debug for NameSection<B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        todo!()
    }
}
