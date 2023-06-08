use crate::wat;

impl<B: crate::bytes::Bytes> wat::Wat for crate::component::TagsComponent<B> {
    fn write(self, w: &mut wat::Writer) -> wat::Parsed<()> {
        let mut tags = self.borrowed();

        loop {
            let result = tags
                .parse(|crate::component::Tag::Exception(tag)| wat::Parsed::Ok(*tag))
                .transpose();

            match result {
                None => return Ok(()),
                Some(tag) => {
                    w.open_paren();
                    w.write_str("tag ");
                    wat::write_type_use(tag?, w);
                    w.close_paren();
                    writeln!(w);
                }
            }
        }
    }
}
