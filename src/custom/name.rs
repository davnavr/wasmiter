//! Types to parse and describe the structure of the
//! [`name` section, described in the WebAssembly specification appendix](https://webassembly.github.io/spec/core/appendix/custom.html),
//! which associates UTF-8 [`Name`](parser::name::Name)s with definitions in a module.

use crate::{
    bytes::{Bytes, Window},
    index,
    parser::{self, AscendingOrder, ResultExt as _},
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

/// Result type used when interpreting the contents of a [`NameSubsection`].
pub type InterpretedNameSubsection<B> =
    Result<parser::Result<NameSubsection<Window<B>>>, Section<B>>;

impl<B: Bytes> NameSubsection<Window<B>> {
    /// Attempts to interpret the contents of the given name subsection.
    ///
    /// Returns `Err(_)` if the section's
    /// [*id*](https://webassembly.github.io/spec/core/appendix/custom.html#subsections) is not
    /// recognized.
    ///
    /// Returns `Ok(Err(_))` if the section **was** recognized, but an attempt to parse a field
    /// failed.
    pub fn interpret(section: Section<B>) -> InterpretedNameSubsection<B> {
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
///
/// Section IDs are checked to ensure that they are in **ascending** order.
#[derive(Clone, Copy)]
pub struct NameSection<B: Bytes> {
    order: AscendingOrder<u8>,
    first: bool,
    sections: SectionSequence<B>,
}

impl<B: Bytes> NameSection<B> {
    /// Creates a [`NameSection`] from the given sequence of `sections`.
    pub fn new(sections: SectionSequence<B>) -> Self {
        Self {
            order: AscendingOrder::new(),
            first: true,
            sections,
        }
    }

    /// Consumes the [`NameSection`], returning the remaining subsections.
    pub fn into_sections(self) -> SectionSequence<B> {
        self.sections
    }

    fn parse_inner<'a, U: Bytes, F>(&'a mut self, f: F) -> Option<InterpretedNameSubsection<U>>
    where
        U: 'a,
        F: FnOnce(Section<&'a B>) -> Section<U>,
    {
        match self.sections.parse().context("name subsection") {
            Ok(Some(section)) => match self
                .order
                .check(section.id(), self.first)
                .context("invalid name subsection order")
            {
                Ok(_) => {
                    self.first = false;
                    Some(NameSubsection::interpret(f(section)))
                }
                Err(e) => Some(Ok(Err(e))),
            },
            Ok(None) => None,
            Err(e) => Some(Ok(Err(e))),
        }
    }

    /// Attempts to parse the next subsection, returning `None` if no more remain.
    ///
    /// If a subsection is not recognized, `Some(Err(_))` is returned.
    ///
    /// # Errors
    ///
    /// `Some(Ok(Err(_)))` is returned for any parser errors or if section IDs are not in **ascending order**.
    pub fn parse(&mut self) -> Option<InterpretedNameSubsection<&B>> {
        self.parse_inner(core::convert::identity)
    }
}

impl<B: Clone + Bytes> Iterator for NameSection<B> {
    type Item = InterpretedNameSubsection<B>;

    fn next(&mut self) -> Option<Self::Item> {
        self.parse_inner(|subsec| subsec.cloned())
    }
}

impl<B: Bytes> Debug for NameSection<B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut list = f.debug_list();
        for subsec in self.sections.borrowed() {
            let err;
            let unknown_subsection;
            let known_subsection;

            list.entry(match subsec {
                Ok(section) => match NameSubsection::interpret(section) {
                    Ok(Err(e)) => {
                        err = parser::Result::<()>::Err(e);
                        &err
                    }
                    Ok(Ok(known)) => {
                        known_subsection = parser::Result::Ok(known);
                        &known_subsection
                    }
                    Err(unknown) => {
                        unknown_subsection = parser::Result::Ok(unknown);
                        &unknown_subsection
                    }
                },
                Err(e) => {
                    err = parser::Result::<()>::Err(e);
                    &err
                }
            });
        }
        list.finish()
    }
}
