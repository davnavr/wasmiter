//! Types to parse and describe the structure of the
//! [`name` section, described in the WebAssembly specification appendix](https://webassembly.github.io/spec/core/appendix/custom.html),
//! which associates UTF-8 [`Name`](parser::name::Name)s with definitions in a module.

use crate::{
    index,
    input::{BorrowInput, CloneInput, HasInput, Input, Window},
    parser::{self, AscendingOrder, ResultExt as _},
    sections::{Section, SectionSequence},
};
use core::fmt::Debug;

mod indirect_name_map;
mod name_assoc;
mod name_map;

pub use indirect_name_map::IndirectNameMap;
pub use name_assoc::NameAssoc;
pub use name_map::NameMap;

const MODULE_NAME_ID: u8 = 0;
const FUNCTION_NAME_ID: u8 = 1;
const LOCAL_NAME_ID: u8 = 2;
const TAG_NAME_ID: u8 = 11;

/// Represents a
/// [name subsection](https://webassembly.github.io/spec/core/appendix/custom.html#subsections)
/// within the [`NameSection`].
#[derive(Clone, Copy)]
#[non_exhaustive]
pub enum NameSubsection<I: Input> {
    /// The
    /// [*module name subsection*](https://webassembly.github.io/spec/core/appendix/custom.html#module-names)
    /// assigns a name to the WebAssembly module.
    ModuleName(parser::name::Name<I>),
    /// The
    /// [*function name subsection*](https://webassembly.github.io/spec/core/appendix/custom.html#function-names)
    /// assigns names to the
    /// [functions](https://webassembly.github.io/spec/core/syntax/modules.html#functions) of a
    /// WebAssembly module.
    FunctionName(NameMap<index::FuncIdx, u64, I>),
    /// The
    /// [*local name subsection*](https://webassembly.github.io/spec/core/appendix/custom.html#local-names)
    /// assigns a [`NameMap`] of local variable names for the functions within a WebAssembly module.
    LocalName(IndirectNameMap<index::FuncIdx, index::LocalIdx, u64, I>),
    /// The
    /// [*tag name subsection*](https://webassembly.github.io/exception-handling/core/appendix/custom.html#tag-names)
    /// assignes names to the
    /// [**tags**](https://webassembly.github.io/exception-handling/core/syntax/modules.html#tags)
    /// of a WebAssembly module.
    ///
    /// Introduced as part of the
    /// [exception handling proposal](https://github.com/WebAssembly/exception-handling).
    TagName(NameMap<index::TagIdx, u64, I>),
}

/// Result type used when interpreting the contents of a [`NameSubsection`].
pub type InterpretedNameSubsection<I> =
    Result<parser::Parsed<NameSubsection<Window<I>>>, Section<I>>;

impl<I: Input> NameSubsection<Window<I>> {
    /// Attempts to interpret the contents of the given name subsection.
    ///
    /// Returns `Err(_)` if the section's
    /// [*id*](https://webassembly.github.io/spec/core/appendix/custom.html#subsections) is not
    /// recognized.
    ///
    /// Returns `Ok(Err(_))` if the section **was** recognized, but an attempt to parse a field
    /// failed.
    pub fn interpret(section: Section<I>) -> InterpretedNameSubsection<I> {
        match section.id() {
            MODULE_NAME_ID => {
                let contents = section.into_contents();
                let mut offset = contents.base();
                Ok(parser::name::parse(&mut offset, contents).map(Self::ModuleName))
            }
            FUNCTION_NAME_ID => {
                let contents = section.into_contents();
                Ok(NameMap::new(contents.base(), contents).map(Self::FunctionName))
            }
            LOCAL_NAME_ID => {
                let contents = section.into_contents();
                Ok(IndirectNameMap::new(contents.base(), contents).map(Self::LocalName))
            }
            TAG_NAME_ID => {
                let contents = section.into_contents();
                Ok(NameMap::new(contents.base(), contents).map(Self::TagName))
            }
            _ => Err(section),
        }
    }

    /// Gets the byte
    /// [*id*](https://webassembly.github.io/spec/core/appendix/custom.html#subsections) associated
    /// with the name subsection.
    pub fn id(&self) -> u8 {
        match self {
            Self::ModuleName(_) => MODULE_NAME_ID,
            Self::FunctionName(_) => FUNCTION_NAME_ID,
            Self::LocalName(_) => LOCAL_NAME_ID,
            Self::TagName(_) => TAG_NAME_ID,
        }
    }
}

impl<I: Input> Debug for NameSubsection<I> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ModuleName(name) => f.debug_tuple("ModuleName").field(name).finish(),
            Self::FunctionName(names) => f.debug_tuple("FunctionName").field(names).finish(),
            Self::LocalName(names) => f.debug_tuple("LocalName").field(names).finish(),
            Self::TagName(names) => f.debug_tuple("TagName").field(names).finish(),
        }
    }
}

/// Represents the sequence of subsections in the
/// [`name` section](https://webassembly.github.io/spec/core/appendix/custom.html).
///
/// Section IDs are checked to ensure that they are in **ascending** order.
#[derive(Clone, Copy)]
pub struct NameSection<I: Input> {
    order: AscendingOrder<u8>,
    first: bool,
    sections: SectionSequence<I>,
}

impl<I: Input> NameSection<I> {
    /// Creates a [`NameSection`] from the given sequence of `sections`.
    pub fn new(sections: SectionSequence<I>) -> Self {
        Self {
            order: AscendingOrder::new(),
            first: true,
            sections,
        }
    }

    /// Consumes the [`NameSection`], returning the remaining subsections.
    pub fn into_sections(self) -> SectionSequence<I> {
        self.sections
    }

    fn parse_inner<'a, U: Input, F>(&'a mut self, f: F) -> Option<InterpretedNameSubsection<U>>
    where
        U: 'a,
        F: FnOnce(Section<&'a I>) -> Section<U>,
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
    #[inline]
    pub fn parse(&mut self) -> Option<InterpretedNameSubsection<&I>> {
        self.parse_inner(core::convert::identity)
    }
}

impl<I: Input> HasInput<I> for NameSection<I> {
    #[inline]
    fn input(&self) -> &I {
        self.sections.input()
    }
}

impl<'a, I: Input + 'a> BorrowInput<'a, I> for NameSection<I> {
    type Borrowed = NameSection<&'a I>;

    #[inline]
    fn borrow_input(&'a self) -> Self::Borrowed {
        NameSection {
            order: self.order,
            first: self.first,
            sections: self.sections.borrow_input(),
        }
    }
}

impl<'a, I: Clone + Input + 'a> CloneInput<'a, I> for NameSection<&'a I> {
    type Cloned = NameSection<I>;

    #[inline]
    fn clone_input(&self) -> Self::Cloned {
        NameSection {
            order: self.order,
            first: self.first,
            sections: self.sections.clone_input(),
        }
    }
}

impl<I: Clone + Input> Iterator for NameSection<I> {
    type Item = InterpretedNameSubsection<I>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.parse_inner(|s| s.clone_input())
    }
}

impl<I: Input> Debug for NameSection<I> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut list = f.debug_list();
        for subsec in self.sections.borrow_input() {
            let err;
            let unknown_subsection;
            let known_subsection;

            list.entry(match subsec {
                Ok(section) => match NameSubsection::interpret(section) {
                    Ok(Err(e)) => {
                        err = parser::Parsed::<()>::Err(e);
                        &err
                    }
                    Ok(Ok(known)) => {
                        known_subsection = parser::Parsed::Ok(known);
                        &known_subsection
                    }
                    Err(unknown) => {
                        unknown_subsection = parser::Parsed::Ok(unknown);
                        &unknown_subsection
                    }
                },
                Err(e) => {
                    err = parser::Parsed::<()>::Err(e);
                    &err
                }
            });
        }
        list.finish()
    }
}
