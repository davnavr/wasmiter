use crate::wat;

impl<B: crate::bytes::Bytes> wat::Wat for crate::component::MemsComponent<B> {
    fn write(self, w: &mut wat::Writer) -> crate::parser::Result<()> {
        for result in self {
            let mem = result?;
            w.open_paren();
            w.write_str("memory ");
            wat::write_mem_type(&mem, w);
            w.close_paren();
            writeln!(w);
        }

        Ok(())
    }
}
