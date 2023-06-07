use crate::{component::DataMode, wat};

impl<B: crate::bytes::Bytes> core::fmt::Display for crate::component::DatasComponent<B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut w = wat::Writer::new(f);
        let mut datas_component = self.borrowed();

        for i in (0u32..).flat_map(crate::index::DataIdx::try_from) {
            w.write_str("(data ");
            wat::write_index(true, i, &mut w);
            w.write_char(' ');

            let writer = core::cell::RefCell::new(&mut w);
            let result = datas_component.parse(
                move |m| match m {
                    DataMode::Passive => crate::parser::Result::Ok(writer),
                    DataMode::Active(memory, offset) => {
                        let mut w = writer.borrow_mut();
                        w.write_str("(memory ");
                        wat::write_index(false, *memory, &mut w);
                        w.write_str(") (offset ");
                        wat::instruction_text::expression_linear(offset.borrowed(), &mut w);
                        w.write_char(')');
                        core::mem::drop(w);
                        crate::parser::Result::Ok(writer)
                    }
                },
                |writer, data| {
                    let mut w = writer.borrow_mut();
                    write!(w, "\"TODO: TODO: Print bytes {data:?}\"");
                    crate::parser::Result::Ok(())
                },
            );

            w.write_char(')');

            match result {
                Ok(Some(())) => (),
                Ok(None) | Err(_) => break,
            }
        }

        w.finish()
    }
}
