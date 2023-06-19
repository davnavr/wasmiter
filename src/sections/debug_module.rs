use crate::{
    input::{BorrowInput, Input},
    sections::{Section, SectionSequence},
};
use core::fmt::Debug;

/// Helper struct to display a WebAssembly module section.
///
/// Returned by the [`Section::debug_module`] method.
pub struct DebugModuleSection<'a, I: Input> {
    section: &'a Section<I>,
}

impl<'a, I: Input> DebugModuleSection<'a, I> {
    pub(super) fn new(section: &'a Section<I>) -> Self {
        Self { section }
    }
}

impl<I: Input> Debug for DebugModuleSection<'_, I> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match crate::component::KnownSection::interpret(self.section.borrow_input()) {
            Ok(known) => Debug::fmt(&known, f),
            Err(unknown) => match crate::custom::CustomSection::try_from_section(unknown) {
                Ok(Err(e)) => Debug::fmt(&crate::parser::Parsed::<()>::Err(e), f),
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
pub struct DebugModule<'a, I: Input> {
    sections: &'a SectionSequence<I>,
}

impl<'a, I: Input> DebugModule<'a, I> {
    pub(super) fn new(sections: &'a SectionSequence<I>) -> Self {
        Self { sections }
    }
}

impl<I: Input> Debug for DebugModule<'_, I> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut list = f.debug_list();
        for result in self.sections.borrow_input() {
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
