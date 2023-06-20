use crate::wat;

impl<B: crate::input::Input> wat::Wat for crate::component::TablesComponent<B> {
    fn write(self, w: &mut wat::Writer) -> wat::Result {
        for result in self {
            w.open_paren()?;
            w.write_str("table ")?;
            wat::write_table_type(&result?, w)?;
            w.close_paren()?;
            writeln!(w)?;
        }

        Ok(())
    }
}
