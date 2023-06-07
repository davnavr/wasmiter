use crate::{
    component::KnownSection,
    sections::SectionKind,
    wat::{self, Wat},
};

impl<B: crate::bytes::Bytes> Wat for crate::sections::SectionSequence<B> {
    fn write(self, w: &mut wat::Writer) -> wat::Parsed<()> {
        for result in self.borrowed() {
            match KnownSection::try_from_section(result?) {
                Ok(known) => match known? {
                    KnownSection::Type(types) => Wat::write(types, w)?,
                    KnownSection::Import(imports) => Wat::write(imports, w)?,
                    //Function
                    KnownSection::Table(tables) => Wat::write(tables, w)?,
                    KnownSection::Memory(mems) => Wat::write(mems, w)?,
                    KnownSection::Global(globals) => Wat::write(globals, w)?,
                    KnownSection::Export(exports) => Wat::write(exports, w)?,
                    KnownSection::Start(start) => wat::write_index(false, start, w),
                    KnownSection::Element(elems) => Wat::write(elems, w)?,
                    //Code
                    KnownSection::Data(data) => Wat::write(data, w)?,
                    KnownSection::DataCount(count) => write!(w, ";; data count = {count}"),
                    bad => todo!("display {bad:?}"),
                },
                Err(section) => {
                    write!(w, "(; ");
                    match section.kind() {
                        SectionKind::Custom(custom) => write!(w, "{custom:?} (custom)"),
                        SectionKind::Id(id) => write!(w, "{id}"),
                    }
                    writeln!(w, " section @ {:#X}", section.contents().base());
                    writeln!(
                        w,
                        "{:?}",
                        crate::bytes::DebugBytes::from(section.into_contents())
                    );
                    w.write_str(";)");
                }
            }

            writeln!(w);
        }

        Ok(())
    }
}
