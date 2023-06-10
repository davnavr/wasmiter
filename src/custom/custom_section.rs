use crate::{
    bytes::{Bytes, Window},
    parser::{self, name::Name},
    sections::{self, Section},
};
use core::fmt::Debug;

/// Represents a
/// [*custom section*](https://webassembly.github.io/spec/core/appendix/custom.html) in a
/// WebAssembly module.
#[derive(Clone, Copy)]
pub struct CustomSection<B: Bytes> {
    name: Name<B>,
    contents: Window<B>,
}

impl<B: Clone + Bytes> CustomSection<B> {
    /// Attempts to interpret the given [`Section`] as a WebAssembly *custom section*.
    ///
    /// Returns `Ok(Err(_))` if the section **was** a custom section, but an error occured while
    /// parsing the section name.
    ///
    /// Returns `Err(_)` if the section is **not** a custom section.
    pub fn try_from_section(section: Section<B>) -> Result<parser::Result<Self>, Section<B>> {
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
                contents: Window::new(
                    contents.into_inner(),
                    new_base,
                    old_length - (new_base - old_base),
                ),
            }),
            Err(e) => Err(e),
        })
    }
}

impl<B: Bytes> CustomSection<B> {
    /// Gets the name of the custom section.
    #[inline]
    pub fn name(&self) -> &Name<B> {
        &self.name
    }

    /// Gets the contents of the custom section (after the custom section name).
    #[inline]
    pub fn contents(&self) -> &Window<B> {
        &self.contents
    }

    /// Borrows the underlying contents of the custom section.
    pub fn borrowed(&self) -> CustomSection<&B> {
        CustomSection {
            name: self.name.borrowed(),
            contents: self.contents.borrowed(),
        }
    }

    /// Consumes the [`CustomSection`], returning its contents.
    pub fn into_contents(self) -> Window<B> {
        self.contents
    }
}

impl<B: Clone + Bytes> CustomSection<&B> {
    /// Clones the underlying contents of the custom section.
    pub fn cloned(&self) -> CustomSection<B> {
        CustomSection {
            name: self.name.really_cloned(),
            contents: self.contents.cloned(),
        }
    }
}

impl<B: Bytes> Debug for CustomSection<B> {
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
