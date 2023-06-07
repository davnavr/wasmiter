use crate::wat;

impl<B: crate::bytes::Bytes> wat::Wat for crate::component::MemsComponent<B> {
    fn write(self, w: &mut wat::Writer) -> crate::parser::Result<()> {
        for (i, result) in (0u32..).flat_map(crate::index::MemIdx::try_from).zip(self) {
            let mem = result?;
            w.open_paren();
            w.write_str("memory ");
            wat::write_index(true, i, w);
            w.write_char(' ');
            wat::write_mem_type(&mem, w);
            w.close_paren();
            writeln!(w);
        }

        Ok(())
    }
}
