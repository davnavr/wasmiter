use crate::{
    bytes::Bytes,
    component,
    wat::{self, Wat, Writer},
};
use core::fmt::{Display, Formatter, Result};

fn write_wat(thing: impl Wat, f: &mut Formatter) -> Result {
    let mut writer = Writer::new(f);
    if let Err(e) = thing.write(&mut writer) {
        wat::write_err(&e, &mut writer);
    }
    writer.finish()
}

impl<T: Bytes, C: Bytes> Display for component::FuncsComponent<T, C> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write_wat(self.borrowed(), f)
    }
}

impl<B: Bytes> Display for component::TypesComponent<B> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write_wat(self.borrowed(), f)
    }
}

impl<B: Bytes> Display for component::ImportsComponent<B> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write_wat(self.borrowed(), f)
    }
}

impl<B: Bytes> Display for component::TablesComponent<B> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write_wat(self.borrowed(), f)
    }
}

impl<B: Bytes> Display for component::MemsComponent<B> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write_wat(self.borrowed(), f)
    }
}

impl<B: Bytes> Display for component::GlobalsComponent<B> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write_wat(self.borrowed(), f)
    }
}

impl<B: Bytes> Display for component::ExportsComponent<B> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write_wat(self.borrowed(), f)
    }
}

impl<B: Bytes> Display for component::ElemsComponent<B> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write_wat(self.borrowed(), f)
    }
}

impl<B: Bytes> Display for component::DatasComponent<B> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write_wat(self.borrowed(), f)
    }
}

//impl Display for Instruction // needs borrowed() method

impl<O: crate::parser::Offset, B: Bytes> Display
    for crate::instruction_set::InstructionSequence<O, B>
{
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write_wat(self.borrowed(), f)
    }
}

impl<B: Bytes> Display for crate::sections::SectionSequence<B> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write_wat(self.borrowed(), f)
    }
}