use crate::wat;

impl<B: crate::input::Input> wat::Wat for crate::component::TypesComponent<B> {
    fn write(mut self, mut w: &mut wat::Writer) -> wat::Result {
        for i in (0u32..).flat_map(crate::index::TypeIdx::try_from) {
            let result = self.parse_mixed(
                move |params| {
                    w.open_paren()?;
                    w.write_str("type ")?;
                    wat::write_index(true, i, w)?;
                    w.write_char(' ')?;
                    w.open_paren()?;
                    w.write_str("func ")?;
                    w.open_paren()?;
                    w.write_str("param")?;
                    wat::write_types(params, w)?;
                    w.close_paren()?;
                    Ok(w)
                },
                |w, results| {
                    w.write_char(' ')?;
                    w.open_paren()?;
                    w.write_str("result")?;
                    wat::write_types(results, w)?;
                    w.close_paren()?;
                    w.close_paren()?; // func
                    w.close_paren()?; // type
                    writeln!(w)?;
                    Ok(w)
                },
            )?;

            match result {
                Some(wr) => {
                    w = wr;
                }
                None => break,
            }
        }

        Ok(())
    }
}
