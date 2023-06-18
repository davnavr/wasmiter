use crate::wat;

impl<B: crate::input::Input> wat::Wat for crate::component::TablesComponent<B> {
    fn write(self, w: &mut wat::Writer) -> crate::parser::Parsed<()> {
        for (i, result) in (0u32..).flat_map(crate::index::MemIdx::try_from).zip(self) {
            let table = result?;
            w.open_paren();
            w.write_str("table ");
            wat::write_index(true, i, w);
            w.write_char(' ');
            wat::write_table_type(&table, w);
            w.close_paren();
            writeln!(w);
        }

        Ok(())
    }
}
