use crate::input::Input;
use crate::{component::DataMode, wat};

impl<I: Input> wat::Wat for crate::component::DatasComponent<I> {
    fn write(mut self, mut w: &mut wat::Writer) -> wat::Parsed<()> {
        for i in (0u32..).flat_map(crate::index::DataIdx::try_from) {
            let result = self.parse(
                move |m| {
                    w.open_paren();
                    w.write_str("data ");
                    wat::write_index(true, i, w);
                    match m {
                        DataMode::Passive => Ok(w),
                        DataMode::Active(memory, offset) => {
                            if memory.to_u32() != 0 {
                                w.write_char(' ');
                                w.open_paren();
                                w.write_str("memory ");
                                wat::write_index(false, *memory, w);
                                w.close_paren();
                            }

                            w.write_char(' ');
                            w.open_paren();
                            w.write_str("offset ");
                            wat::instruction_text::expression_linear(offset, w)?;
                            w.close_paren();
                            Ok(w)
                        }
                    }
                },
                |w: &mut wat::Writer, data| {
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
                            data.read_exact(&mut offset, &mut buffer[..buffer_size])?;
                            length -= buffer_size;

                            w.write_char('"');
                            for b in &buffer[..buffer_size] {
                                match b {
                                    0x20..=0x21 | 0x23..=0x26 | 0x28..=0x5B | 0x5D..=0x7E => {
                                        w.write_char(char::from_u32(u32::from(*b)).unwrap())
                                    }
                                    b'\t' => w.write_str("\\t"),
                                    b'\n' => w.write_str("\\n"),
                                    b'\r' => w.write_str("\\r"),
                                    b'\'' => w.write_str("\\'"),
                                    b'\"' => w.write_str("\\\""),
                                    b'\\' => w.write_str("\\\\"),
                                    _ => write!(w, "\\{b:02X}"),
                                }
                            }

                            w.write_char('"');

                            // Write indentation for next line if there are more bytes to write
                            if length > 0 {
                                writeln!(w);
                                w.write_str(wat::INDENTATION);
                            }
                        }
                    } else {
                        w.write_str("\"\"");
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
