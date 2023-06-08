use crate::{component::ExportKind, wat};

impl<B: crate::bytes::Bytes> wat::Wat for crate::component::ExportsComponent<B> {
    fn write(self, w: &mut wat::Writer) -> wat::Parsed<()> {
        for result in self.borrowed() {
            w.open_paren();
            w.write_str("export ");
            let export = result?;
            write!(w, "{:?} ", export.name());
            w.open_paren();
            match export.kind() {
                ExportKind::Function(idx) => {
                    w.write_str("func ");
                    wat::write_index(false, *idx, w)
                }
                ExportKind::Table(idx) => {
                    w.write_str("table ");
                    wat::write_index(false, *idx, w)
                }
                ExportKind::Memory(idx) => {
                    w.write_str("memory ");
                    wat::write_index(false, *idx, w)
                }
                ExportKind::Global(idx) => {
                    w.write_str("global ");
                    wat::write_index(false, *idx, w)
                }
                ExportKind::Tag(idx) => {
                    w.write_str("tag ");
                    wat::write_index(false, *idx, w)
                }
            }
            w.close_paren();
            w.close_paren();
            writeln!(w);
        }

        Ok(())
    }
}
