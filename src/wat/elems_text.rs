use crate::{bytes::Bytes, component::ElementInit, component::ElementMode, wat};

impl<B: Bytes> wat::Wat for crate::component::ElemsComponent<B> {
    fn write(mut self, mut w: &mut wat::Writer) -> wat::Parsed<()> {
        for i in (0u32..).flat_map(crate::index::ElemIdx::try_from) {
            let result = self.parse(
                |mode| {
                    w.open_paren();
                    w.write_str("elem ");
                    wat::write_index(true, i, w);
                    w.write_char(' ');

                    match mode {
                        ElementMode::Passive => (),
                        ElementMode::Declarative => w.write_str("declare"),
                        ElementMode::Active(table, offset) => {
                            write!(w, "(table ");
                            wat::write_index(false, *table, w);
                            w.write_str(") ");
                            w.open_paren();
                            w.write_str("offset ");
                            wat::instruction_text::expression_linear(offset, w)?;
                            w.close_paren();
                        }
                    }

                    w.write_char(' ');
                    Ok(w)
                },
                |mut w, init| {
                    match init {
                        ElementInit::Functions(functions) => {
                            w.write_str("func");
                            for idx in functions {
                                w.write_char(' ');
                                wat::write_index(false, idx?, w);
                            }
                        }
                        ElementInit::Expressions(ref_type, expressions) => {
                            write!(w, "{ref_type} ");

                            let writer = core::cell::RefCell::new(w);

                            loop {
                                let result = expressions.next(|item| {
                                    let mut w = writer.borrow_mut();
                                    w.open_paren();
                                    w.write_str("item ");
                                    wat::instruction_text::expression_linear(item, &mut w)?;
                                    w.close_paren();
                                    Ok(())
                                })?;

                                match result {
                                    Some(()) => (),
                                    None => break,
                                }
                            }

                            w = writer.into_inner();
                        }
                    }

                    w.close_paren();
                    writeln!(w);
                    Ok(w)
                },
            )?;

            match result {
                Some(wr) => w = wr,
                None => break,
            }
        }

        Ok(())
    }
}
