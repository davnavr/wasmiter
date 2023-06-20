use crate::wat;

impl<B: crate::input::Input> wat::Wat for crate::component::MemsComponent<B> {
    fn write(self, w: &mut wat::Writer) -> wat::Result {
        for result in self {
            w.open_paren()?;
            w.write_str("memory ")?;
            wat::write_mem_type(&result?, w)?;
            w.close_paren()?;
            writeln!(w)?;
        }

        Ok(())
    }
}
