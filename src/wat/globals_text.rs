use crate::wat;

impl<B: crate::bytes::Bytes> wat::Wat for crate::component::GlobalsComponent<B> {
    fn write(mut self, mut w: &mut wat::Writer) -> wat::Parsed<()> {
        loop {
            let result = self.parse(move |global_type, init| {
                w.open_paren();
                w.write_str("global ");
                wat::write_global_type(global_type, w);
                wat::instruction_text::expression_linear(init, w)?;
                w.close_paren();
                writeln!(w);
                Ok(w)
            })?;

            match result {
                None => return Ok(()),
                Some(wr) => w = wr,
            }
        }
    }
}
