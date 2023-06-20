use crate::{
    component,
    input::{BorrowInput as _, Input},
    wat::{Wat, Writer},
};
use core::fmt::{Display, Formatter, Result};

fn write_wat(thing: impl Wat, f: &mut Formatter) -> Result {
    let mut writer = Writer::new(f);
    let result = thing.write(&mut writer);
    writer.finish(result)
}

impl<T: Input, C: Input> Display for component::FuncsComponent<T, C> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write_wat(self.borrow_input(), f)
    }
}

impl<I: Input> Display for component::TypesComponent<I> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write_wat(self.borrow_input(), f)
    }
}

impl<I: Input> Display for component::ImportsComponent<I> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write_wat(self.borrow_input(), f)
    }
}

impl<I: Input> Display for component::TablesComponent<I> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write_wat(self.borrow_input(), f)
    }
}

impl<I: Input> Display for component::MemsComponent<I> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write_wat(self.borrow_input(), f)
    }
}

impl<I: Input> Display for component::GlobalsComponent<I> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write_wat(self.borrow_input(), f)
    }
}

impl<I: Input> Display for component::ExportsComponent<I> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write_wat(self.borrow_input(), f)
    }
}

impl<I: Input> Display for component::ElemsComponent<I> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write_wat(self.borrow_input(), f)
    }
}

impl<I: Input> Display for component::DatasComponent<I> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write_wat(self.borrow_input(), f)
    }
}

impl<I: Input> Display for component::TagsComponent<I> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write_wat(self.borrow_input(), f)
    }
}

//impl Display for Instruction // needs borrow_input() method

impl<O: crate::parser::Offset, I: Input> Display
    for crate::instruction_set::InstructionSequence<O, I>
{
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write_wat(self.borrow_input(), f)
    }
}

impl<I: Input> Display for crate::sections::DisplayModule<'_, I> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write_wat(Self::new(self.as_sections()), f)
    }
}
