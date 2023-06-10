//! Types and functions for parsing the contents of
//! [WebAssembly custom sections](https://webassembly.github.io/spec/core/appendix/custom.html).

use crate::{
    bytes::{Bytes, Window},
    sections::{id as section_id, SectionSequence},
};
use core::fmt::Debug;

mod custom_section;

pub mod name;

pub use custom_section::CustomSection;

/// Represents a well-known
/// [custom section](https://webassembly.github.io/spec/core/appendix/custom.html) in a WebAssembly
/// module.
#[derive(Clone, Copy)]
#[non_exhaustive]
#[allow(missing_docs)]
pub enum KnownCustomSection<B: Bytes> {
    Name(name::NameSection<B>),
}

impl<B: Bytes> KnownCustomSection<Window<B>> {
    /// Attempts to interpret the contents of the given [`CustomSection`].
    ///
    /// # Errors
    ///
    /// Returns `section` if it was not recognized.
    pub fn interpret(section: CustomSection<B>) -> Result<Self, CustomSection<B>> {
        if let Some(static_name) = section_id::is_custom_name_recognized(section.name().borrowed())
        {
            match static_name {
                section_id::NAME => {
                    let contents = section.into_contents();
                    Ok(Self::Name(name::NameSection::new(SectionSequence::new(
                        contents.base(),
                        contents,
                    ))))
                }
                _ => Err(section),
            }
        } else {
            Err(section)
        }
    }

    /// Gets the name of the custom section.
    pub fn name(&self) -> &str {
        section_id::NAME
    }
}

impl<B: Bytes> Debug for KnownCustomSection<B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Name(names) => Debug::fmt(names, f),
        }
    }
}
