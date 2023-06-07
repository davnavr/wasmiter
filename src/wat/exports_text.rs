use crate::component::ExportKind;
use crate::wat;

impl<B: crate::bytes::Bytes> core::fmt::Display for crate::component::ExportsComponent<B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut w = wat::Writer::new(f);
        for result in self.borrowed() {
            w.write_str("(export ");
            match result {
                Ok(export) => {
                    write!(w, "\"{}\" (", export.name());
                    match export.kind() {
                        ExportKind::Function(idx) => {
                            w.write_str("func ");
                            wat::write_index(false, *idx, &mut w)
                        }
                        ExportKind::Table(idx) => {
                            w.write_str("table ");
                            wat::write_index(false, *idx, &mut w)
                        }
                        ExportKind::Memory(idx) => {
                            w.write_str("memory ");
                            wat::write_index(false, *idx, &mut w)
                        }
                        ExportKind::Global(idx) => {
                            w.write_str("global ");
                            wat::write_index(false, *idx, &mut w)
                        }
                    }
                    w.write_char(')')
                }
                Err(e) => wat::write_err(&e, &mut w),
            }
            writeln!(w, ")");
        }

        w.finish()
    }
}
