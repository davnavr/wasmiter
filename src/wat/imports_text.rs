use crate::{component::ImportKind, index, wat};

impl<B: crate::bytes::Bytes> core::fmt::Display for crate::component::ImportsComponent<B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut w = wat::Writer::new(f);
        let mut function_count = 0u32;
        let mut table_count = 0u32;
        let mut memory_count = 0u32;
        let mut global_count = 0u32;
        for result in self.borrowed() {
            w.write_str("(import ");
            match result {
                Ok(import) => {
                    write!(w, "{:?} {:?} (", import.module(), import.name());
                    match import.kind() {
                        ImportKind::Function(ty) => {
                            w.write_str("func ");
                            wat::write_index(
                                true,
                                index::FuncIdx::try_from(function_count).unwrap(),
                                &mut w,
                            );
                            w.write_char(' ');
                            wat::write_type_use(*ty, &mut w);
                            function_count += 1;
                        }
                        ImportKind::Table(ty) => {
                            w.write_str("table ");
                            wat::write_index(
                                true,
                                index::TableIdx::try_from(table_count).unwrap(),
                                &mut w,
                            );
                            w.write_char(' ');
                            wat::write_table_type(ty, &mut w);
                            table_count += 1;
                        }
                        ImportKind::Memory(ty) => {
                            w.write_str("memory ");
                            wat::write_index(
                                true,
                                index::MemIdx::try_from(memory_count).unwrap(),
                                &mut w,
                            );
                            w.write_char(' ');
                            wat::write_mem_type(ty, &mut w);
                            memory_count += 1;
                        }
                        ImportKind::Global(ty) => {
                            w.write_str("global ");
                            wat::write_index(
                                true,
                                index::GlobalIdx::try_from(global_count).unwrap(),
                                &mut w,
                            );
                            w.write_char(' ');
                            wat::write_global_type(*ty, &mut w);
                            global_count += 1;
                        }
                    }
                    w.write_char(')');
                }
                Err(e) => wat::write_err(&e, &mut w),
            }
            writeln!(w, ")");
        }

        w.finish()
    }
}
