use crate::bytes::Bytes;
use crate::{component::DataMode, wat};

impl<B: Bytes> core::fmt::Display for crate::component::DatasComponent<B> {
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
                    let mut length = usize::try_from(data.length()).unwrap_or(usize::MAX);

                    if length > 0 {
                        if length > 16 {
                            writeln!(w);
                            w.write_str(wat::INDENTATION);
                        } else {
                            w.write_char(' ');
                        }

                        let mut buffer = [0u8; 16];
                        let mut offset = data.base();

                        while length > 0 {
                            let buffer_size = core::cmp::min(buffer.len(), length);

                            if let Err(e) = data.read_exact(&mut offset, &mut buffer[..buffer_size])
                            {
                                wat::write_err(&e.into(), &mut w);
                                break;
                            } else {
                                length -= buffer_size;
                            }

                            w.write_char('"');
                            for b in &buffer[..buffer_size] {
                                write!(w, "{}", core::ascii::escape_default(*b));

                                // Write indentation for next line if there are more bytes to write
                                if length > 0 {
                                    writeln!(w);
                                    w.write_str(wat::INDENTATION);
                                }
                            }
                            w.write_char('"');
                        }
                    } else {
                        w.write_str("\"\"");
                    }

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
