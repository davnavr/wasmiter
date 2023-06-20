use crate::{component::Tag, input::BorrowInput as _, wat};

pub(super) fn write_tag(Tag::Exception(tag): Tag, w: &mut wat::Writer) -> wat::Result {
    wat::write_type_use(tag, w)
}

impl<B: crate::input::Input> wat::Wat for crate::component::TagsComponent<B> {
    fn write(self, w: &mut wat::Writer) -> wat::Result {
        for result in self.borrow_input() {
            w.open_paren()?;
            w.write_str("tag ")?;
            write_tag(result?, w)?;
            w.close_paren()?;
            writeln!(w)?;
        }

        Ok(())
    }
}
