use crate::{
    input::{BorrowInput, CloneInput, HasInput, Input, Window},
    parser::{self, name::Name},
    sections::{self, Section},
};
use core::fmt::Debug;

/// Represents a
/// [*custom section*](https://webassembly.github.io/spec/core/appendix/custom.html) in a
/// WebAssembly module.
#[derive(Clone, Copy)]
pub struct CustomSection<I: Input> {
    name: Name<I>,
    contents: Window<I>,
}

impl<I: Clone + Input> CustomSection<I> {
    /// Attempts to interpret the given [`Section`] as a WebAssembly *custom section*.
    ///
    /// Returns `Ok(Err(_))` if the section **was** a custom section, but an error occured while
    /// parsing the section name.
    ///
    /// Returns `Err(_)` if the section is **not** a custom section.
    pub fn try_from_section(section: Section<I>) -> Result<parser::Result<Self>, Section<I>> {
        if section.id() != sections::id::CUSTOM {
            return Err(section);
        }

        let contents = section.into_contents();
        let old_base = contents.base();
        let old_length = contents.length();
        let mut new_base = old_base;

        Ok(match parser::name::parse(&mut new_base, &contents) {
            Ok(name) => Ok(Self {
                name: name.clone_input().into(),
                contents: Window::with_offset_and_length(
                    contents.into_inner(),
                    new_base,
                    old_length - (new_base - old_base),
                ),
            }),
            Err(e) => Err(e),
        })
    }
}

impl<I: Input> CustomSection<I> {
    /// Gets the name of the custom section.
    #[inline]
    pub fn name(&self) -> &Name<I> {
        &self.name
    }

    /// Gets the contents of the custom section (after the custom section name).
    #[inline]
    pub fn contents(&self) -> &Window<I> {
        &self.contents
    }

    /// Consumes the [`CustomSection`], returning its contents.
    pub fn into_contents(self) -> Window<I> {
        self.contents
    }
}

impl<I: Input> HasInput<I> for CustomSection<I> {
    #[inline]
    fn input(&self) -> &I {
        self.contents.input()
    }
}

impl<I: Input> HasInput<Window<I>> for CustomSection<I> {
    #[inline]
    fn input(&self) -> &Window<I> {
        &self.contents
    }
}

impl<'a, I: Input + 'a> BorrowInput<'a, I> for CustomSection<I> {
    type Borrowed = CustomSection<&'a I>;

    #[inline]
    fn borrow_input(&'a self) -> Self::Borrowed {
        CustomSection {
            name: self.name.borrow_input(),
            contents: self.contents.borrow_input(),
        }
    }
}

impl<'a, I: Clone + Input + 'a> CloneInput<'a, I> for CustomSection<&'a I> {
    type Cloned = CustomSection<I>;

    #[inline]
    fn clone_input(&self) -> CustomSection<I> {
        CustomSection {
            name: self.name.clone_input(),
            contents: self.contents.clone_input(),
        }
    }
}

impl<I: Input> Debug for CustomSection<I> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match crate::custom::KnownCustomSection::interpret(self.borrow_input()) {
            Ok(known) => Debug::fmt(&known, f),
            Err(_) => f
                .debug_struct("CustomSection")
                .field("name", &self.name)
                .field("contents", &self.contents)
                .finish(),
        }
    }
}
