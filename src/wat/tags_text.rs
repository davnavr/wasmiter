use crate::{component::Tag, wat};

pub(super) fn write_tag(Tag::Exception(tag): Tag, w: &mut wat::Writer) {
    wat::write_type_use(tag, w);
}

impl<B: crate::input::Input> wat::Wat for crate::component::TagsComponent<B> {
    fn write(self, w: &mut wat::Writer) -> wat::Parsed<()> {
        for result in self.borrowed() {
            w.open_paren();
            w.write_str("tag ");
            write_tag(result?, w);
            w.close_paren();
            writeln!(w);
        }

        Ok(())
    }
}
