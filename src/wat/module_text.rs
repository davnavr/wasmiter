use crate::{component::KnownSection, sections::SectionKind, wat};
use core::fmt::Display;

impl<B: crate::bytes::Bytes> Display for crate::sections::SectionSequence<B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut w = wat::Writer::new(f);

        for result in self.borrowed() {
            match result {
                Ok(section) => match KnownSection::try_from_section(section) {
                    Ok(known) => match known {
                        // KnownSection::Type(types) => Display::fmt(&types, f)?,
                        _ => todo!("display {known:?}"),
                    },
                    Err(section) => {
                        write!(w, "(; ");
                        match section.kind() {
                            SectionKind::Custom(custom) => write!(w, "{custom:?} (custom)"),
                            SectionKind::Id(id) => write!(w, "{id}"),
                        }
                        writeln!(w, " section @ {:#X}", section.contents().base());
                        writeln!(
                            w,
                            "{:?}",
                            crate::bytes::DebugBytes::from(section.into_contents())
                        );
                        w.write_str(";)");
                    }
                },
                Err(e) => wat::write_err(&e, &mut w),
            }
            writeln!(w);
        }

        w.finish()
    }
}
