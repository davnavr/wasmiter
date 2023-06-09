//! Types to parse and describe the structure of the
//! [`name` section, described in the WebAssembly specification appendix](https://webassembly.github.io/spec/core/appendix/custom.html),
//! which associates UTF-8 [`Name`]s with definitions in a module.

use crate::{
    bytes::{self, Bytes},
    parser::name::Name,
    sections::{self, Section},
};
use core::fmt::{Debug, Formatter};

/// Represents the sequence of subsections in the
/// [`name` section](https://webassembly.github.io/spec/core/appendix/custom.html).
#[derive(Clone, Copy)]
pub struct NameSection<B: Bytes> {
    sections: sections::SectionSequence<B>,
}

impl<B: Bytes> NameSection<B> {
    /// Creates a [`NameSection`] from the given sequence of `sections`.
    pub fn new(sections: sections::SectionSequence<B>) -> Self {
        Self { sections }
    }
}

impl<B: Bytes> Debug for NameSection<B> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        todo!()
    }
}
