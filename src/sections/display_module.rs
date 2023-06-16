use crate::{input::Input, sections::SectionSequence};

/// Helper struct to display the sections of a WebAssembly module in the
/// [WebAssembly text format](https://webassembly.github.io/spec/core/text/index.html).
///
/// Returned by the [`SectionSequence::display_module`] method.
pub struct DisplayModule<'a, I: Input> {
    sections: &'a SectionSequence<I>,
}

impl<'a, I: Input> DisplayModule<'a, I> {
    pub(crate) fn new(sections: &'a SectionSequence<I>) -> Self {
        Self { sections }
    }

    #[inline]
    pub(crate) fn as_sections(&self) -> &'a SectionSequence<I> {
        self.sections
    }
}

impl<I: Input> core::fmt::Debug for DisplayModule<'_, I> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "\"{}\"", self)
    }
}
