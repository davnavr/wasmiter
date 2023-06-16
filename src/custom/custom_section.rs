use crate::{
    input::{Input, Window},
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
                name: name.really_cloned().flatten_windowed(),
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

    /// Borrows the underlying contents of the custom section.
    pub fn borrowed(&self) -> CustomSection<&I> {
        CustomSection {
            name: self.name.borrowed(),
            contents: (&self.contents).into(),
        }
    }

    /// Consumes the [`CustomSection`], returning its contents.
    pub fn into_contents(self) -> Window<I> {
        self.contents
    }
}

impl<I: Clone + Input> CustomSection<&I> {
    /// Clones the underlying contents of the custom section.
    pub fn cloned(&self) -> CustomSection<I> {
        CustomSection {
            name: self.name.really_cloned(),
            contents: (&self.contents).into(),
        }
    }
}

impl<I: Input> Debug for CustomSection<I> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match crate::custom::KnownCustomSection::interpret(self.borrowed()) {
            Ok(known) => Debug::fmt(&known, f),
            Err(_) => f
                .debug_struct("CustomSection")
                .field("name", &self.name)
                .field("contents", &self.contents)
                .finish(),
        }
    }
}
