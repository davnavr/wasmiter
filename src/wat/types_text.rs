use crate::wat;

impl<B: crate::bytes::Bytes> core::fmt::Display for crate::component::TypesComponent<B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut types = self.borrowed();
        let mut w = wat::Writer::new(f);

        for i in (0u32..).flat_map(crate::index::TypeIdx::try_from) {
            let result = types.parse(
                |params| Ok(params.dereferenced()),
                |params, results| {
                    w.write_str("(type ");
                    wat::write_index(true, i, &mut w);
                    w.write_str(" (func (param");
                    wat::write_types(params, &mut w);
                    w.write_str(") (result");
                    wat::write_types(results, &mut w);
                    w.write_str("))");
                    Ok(())
                },
            );

            if let Err(e) = &result {
                wat::write_err(e, &mut w);
            }

            if let Ok(Some(())) = result {
                writeln!(w, ")");
                continue;
            }

            break;
        }

        w.finish()
    }
}
