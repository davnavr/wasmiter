use crate::{component::Tag, wat};

pub(super) fn write_tag(result: wat::Parsed<Tag>, w: &mut wat::Writer) -> wat::Parsed<()> {
    w.open_paren();
    w.write_str("tag ");
    let crate::component::Tag::Exception(tag) = result?;
    wat::write_type_use(tag, w);
    w.close_paren();
    Ok(())
}

impl<B: crate::bytes::Bytes> wat::Wat for crate::component::TagsComponent<B> {
    fn write(self, w: &mut wat::Writer) -> wat::Parsed<()> {
        for result in self.borrowed() {
            write_tag(result, w)?;
            writeln!(w);
        }

        Ok(())
    }
}
