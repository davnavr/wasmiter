use crate::{
    component::KnownSection,
    input::BorrowInput,
    wat::{self, Wat},
};

impl<B: crate::input::Input> Wat for crate::sections::DisplayModule<'_, B> {
    fn write(self, w: &mut wat::Writer) -> wat::Parsed<()> {
        w.open_paren();
        w.write_str("module");

        let mut function_types = None;

        for result in self.as_sections().borrow_input() {
            writeln!(w);
            match KnownSection::interpret(result?) {
                Ok(known) => match known? {
                    KnownSection::Type(types) => Wat::write(types, w)?,
                    KnownSection::Import(imports) => Wat::write(imports, w)?,
                    KnownSection::Function(functions) => {
                        write!(
                            w,
                            ";; function section count = {}",
                            functions.remaining_count()
                        );
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
                            write!(w, ";; code section count = {}", code.remaining_count());
                        }
                    }
                    KnownSection::Data(data) => Wat::write(data, w)?,
                    KnownSection::DataCount(count) => write!(w, ";; data count = {count}"),
                    KnownSection::Tag(tags) => Wat::write(tags, w)?,
                },
                Err(section) => {
                    let id = section.id();
                    let contents = section.into_contents();
                    writeln!(
                        w,
                        "(; UNRECOGNIZED ({id}) @ {:#X} to {:#X}",
                        contents.base(),
                        contents.base() + contents.length() - 1,
                    );
                    writeln!(w, "{:#}", crate::input::HexDump::from(contents));
                    w.write_str(";)");
                    writeln!(w);
                }
            }
        }

        w.close_paren();
        Ok(())
    }
}
