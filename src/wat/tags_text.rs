use crate::wat;

impl<B: crate::bytes::Bytes> wat::Wat for crate::component::TagsComponent<B> {
    fn write(self, w: &mut wat::Writer) -> wat::Parsed<()> {
        for result in self.borrowed() {
            w.open_paren();
            w.write_str("tag ");
            let crate::component::Tag::Exception(tag) = result?;
            wat::write_type_use(tag, w);
            w.close_paren();
            writeln!(w);
        }

        Ok(())
    }
}
