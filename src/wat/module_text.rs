use crate::{
    component::KnownSection,
    sections::SectionKind,
    wat::{self, Wat},
};

impl<B: crate::bytes::Bytes> Wat for crate::sections::SectionSequence<B> {
    fn write(self, w: &mut wat::Writer) -> wat::Parsed<()> {
        let mut function_types = None;

        for result in self.borrowed() {
            match KnownSection::interpret(result?) {
                Ok(known) => match known? {
                    KnownSection::Type(types) => Wat::write(types, w)?,
                    KnownSection::Import(imports) => Wat::write(imports, w)?,
                    KnownSection::Function(functions) => {
                        write!(w, ";; function section count = {}", functions.len());
                        function_types = Some(functions);
                    }
                    KnownSection::Table(tables) => Wat::write(tables, w)?,
                    KnownSection::Memory(mems) => Wat::write(mems, w)?,
                    KnownSection::Global(globals) => Wat::write(globals, w)?,
                    KnownSection::Export(exports) => Wat::write(exports, w)?,
                    KnownSection::Start(start) => {
                        w.write_str("(start ");
                        wat::write_index(false, start, w);
                        w.write_char(')');
                    }
                    KnownSection::Element(elems) => Wat::write(elems, w)?,
                    KnownSection::Code(code) => {
                        if let Some(types) = function_types.take() {
                            Wat::write(crate::component::FuncsComponent::new(types, code)?, w)?;
                        } else {
                            write!(w, ";; code section count = {}", code.len());
                        }
                    }
                    KnownSection::Data(data) => Wat::write(data, w)?,
                    KnownSection::DataCount(count) => write!(w, ";; data count = {count}"),
                    KnownSection::Tag(tags) => Wat::write(tags, w)?,
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
