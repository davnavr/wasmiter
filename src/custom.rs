//! Types and functions for parsing the contents of
//! [WebAssembly custom sections](https://webassembly.github.io/spec/core/appendix/custom.html).

use crate::{
    bytes::{Bytes, Window},
    parser::{self, name::Name},
    sections::{self, Section},
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
    /// Returns `Ok(Err(_))` if the custom section **was** recognized, but parsing some field
    /// within resulted in an error.
    ///
    /// Returns `Err(_)` if the section was not recognized.
    pub fn interpret(section: Section<B>) -> Result<parser::Result<Self>, Section<B>> {
        // match section.kind() {
        //     sections::SectionKind::Custom(name)
        // }
        todo!()
    }
}

impl<B: Bytes> Debug for KnownCustomSection<B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        /*
        let kind = SectionId::new(id_byte);
        let mut content_length = u64::from(
            parser::leb128::u32(&mut self.offset, &self.bytes).context("section content size")?,
        );

        let id = if let Some(id_number) = kind {
            SectionKind::Id(id_number)
        } else {
            let name_start = self.offset;

            let name = parser::name::parse(&mut self.offset, &self.bytes)
                .context("custom section name")?;

            content_length -= self.offset - name_start;

            SectionKind::Custom(name)
        };

        let contents = Window::new(&self.bytes, self.offset, content_length);
        */
        match self {
            Self::Name(names) => Debug::fmt(names, f),
        }
    }
}
