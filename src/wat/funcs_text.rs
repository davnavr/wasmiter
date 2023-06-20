use crate::{input::Input, wat};

impl<T: Clone + Input, C: Clone + Input> wat::Wat for crate::component::FuncsComponent<T, C> {
    fn write(self, mut w: &mut wat::Writer) -> wat::Parsed<()> {
        for result in self {
            let func = result?;
            w.open_paren();
            w.write_str("func ");
            wat::write_type_use(func.signature(), w);
            let code = func.into_code();
            write!(w, " ;; code size = {}", code.content().length());
            writeln!(w);

            w = code.parse(
                move |locals| {
                    for (i, result) in (0u32..)
                        .flat_map(crate::index::LocalIdx::try_from)
                        .zip(locals)
                    {
                        let local_type = result?;
                        w.write_str(wat::INDENTATION);
                        w.open_paren();
                        w.write_str("local ");
                        wat::write_index(true, i, w);
                        write!(w, " {local_type}");
                        w.close_paren();
                        writeln!(w);
                    }
                    Ok(w)
                },
                |w, code| {
                    wat::instruction_text::expression_indented(code, true, w)?;
                    Ok(w)
                },
            )?;

            w.close_paren();
            writeln!(w);
        }

        Ok(())
    }
}
