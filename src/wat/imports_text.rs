use crate::{component::ImportKind, index, wat};

impl<B: Clone + crate::bytes::Bytes> wat::Wat for crate::component::ImportsComponent<B> {
    fn write(self, w: &mut wat::Writer) -> wat::Parsed<()> {
        let mut function_count = 0u32;
        let mut table_count = 0u32;
        let mut memory_count = 0u32;
        let mut global_count = 0u32;
        let mut tag_count = 0u32;
        for result in self {
            w.open_paren();
            w.write_str("import ");
            let import = result?;
            write!(w, "{:?} {:?} ", import.module(), import.name());
            w.open_paren();
            match import.kind() {
                ImportKind::Function(ty) => {
                    w.write_str("func ");
                    wat::write_index(true, index::FuncIdx::try_from(function_count).unwrap(), w);
                    w.write_char(' ');
                    wat::write_type_use(*ty, w);
                    function_count += 1;
                }
                ImportKind::Table(ty) => {
                    w.write_str("table ");
                    wat::write_index(true, index::TableIdx::try_from(table_count).unwrap(), w);
                    w.write_char(' ');
                    wat::write_table_type(ty, w);
                    table_count += 1;
                }
                ImportKind::Memory(ty) => {
                    w.write_str("memory ");
                    wat::write_index(true, index::MemIdx::try_from(memory_count).unwrap(), w);
                    w.write_char(' ');
                    wat::write_mem_type(ty, w);
                    memory_count += 1;
                }
                ImportKind::Global(ty) => {
                    w.write_str("global ");
                    wat::write_index(true, index::GlobalIdx::try_from(global_count).unwrap(), w);
                    w.write_char(' ');
                    wat::write_global_type(*ty, w);
                    global_count += 1;
                }
                ImportKind::Tag(tag) => {
                    w.write_str("tag ");
                    wat::write_index(true, index::TagIdx::try_from(tag_count).unwrap(), w);
                    w.write_char(' ');
                    wat::tags_text::write_tag(*tag, w);
                    tag_count += 1;
                }
            }
            w.close_paren();
            w.close_paren();
            writeln!(w);
        }

        Ok(())
    }
}
