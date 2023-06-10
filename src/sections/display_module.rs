use crate::{bytes::Bytes, sections::SectionSequence};

/// Helper struct to display the sections of a WebAssembly module in the
/// [WebAssembly text format](https://webassembly.github.io/spec/core/text/index.html).
///
/// Returned by the [`SectionSequence::display_module`] method.
pub struct DisplayModule<'a, B: Bytes> {
    sections: &'a SectionSequence<B>,
}

impl<'a, B: Bytes> DisplayModule<'a, B> {
    pub(crate) fn new(sections: &'a SectionSequence<B>) -> Self {
        Self { sections }
    }

    #[inline]
    pub(crate) fn as_sections(&self) -> &'a SectionSequence<B> {
        self.sections
    }
}

impl<B: Bytes> core::fmt::Debug for DisplayModule<'_, B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "\"{}\"", self)
    }
}
