use crate::{
    bytes::Bytes,
    sections::{Section, SectionSequence},
};
use core::fmt::Debug;

/// Helper struct to display a WebAssembly module section.
///
/// Returned by the [`Section::debug_module`] method.
pub struct DebugModuleSection<'a, B: Bytes> {
    section: &'a Section<B>,
}

impl<'a, B: Bytes> DebugModuleSection<'a, B> {
    pub(super) fn new(section: &'a Section<B>) -> Self {
        Self { section }
    }
}

impl<B: Bytes> Debug for DebugModuleSection<'_, B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match crate::component::KnownSection::interpret(self.section.borrowed()) {
            Ok(known) => Debug::fmt(&known, f),
            Err(unknown) => match crate::custom::CustomSection::try_from_section(unknown) {
                Ok(Err(e)) => Debug::fmt(&crate::parser::Result::<()>::Err(e), f),
                Ok(Ok(custom)) => match crate::custom::KnownCustomSection::interpret(custom) {
                    Ok(known) => Debug::fmt(&known, f),
                    Err(unknown_custom) => Debug::fmt(&unknown_custom, f),
                },
                Err(really_unknown) => Debug::fmt(&really_unknown, f),
            },
        }
    }
}

/// Helper struct to display the sections of a WebAssembly module.
///
/// Returned by the [`SectionSequence::debug_module`] method.
pub struct DebugModule<'a, B: Bytes> {
    sections: &'a SectionSequence<B>,
}

impl<'a, B: Bytes> DebugModule<'a, B> {
    pub(super) fn new(sections: &'a SectionSequence<B>) -> Self {
        Self { sections }
    }
}

impl<B: Bytes> Debug for DebugModule<'_, B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut list = f.debug_list();
        for result in self.sections.borrowed() {
            let section;
            let entry = match result {
                Ok(sec) => {
                    section = sec;
                    Ok(DebugModuleSection::new(&section))
                }
                Err(e) => Err(e),
            };

            list.entry(&entry);
        }
        list.finish()
    }
}
