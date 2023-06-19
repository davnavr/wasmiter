use crate::{
    parser::{self, ResultExt as _},
    sections::SectionSequence,
};
use memmap2::Mmap;
use std::path::Path;

fn parse_module_sections_inner(path: &Path) -> parser::Parsed<SectionSequence<Mmap>> {
    let file = std::fs::File::open(path).with_context(|| {
        let path_buf = Box::<Path>::from(path);
        move |f| write!(f, "could not open file at path {}", path_buf.display())
    })?;

    // Safety: See documentation for Input impl on Mmap on how unsafe behavior is "ignored"
    let binary = unsafe { memmap2::Mmap::map(&file) };

    crate::parse_module_sections(binary.with_context(|| {
        let path_buf = Box::<Path>::from(path);
        move |f| write!(f, "could not open file at path {}", path_buf.display())
    })?)
}

/// Opens a memory-mapped file containing a
/// [WebAssembly module binary](https://webassembly.github.io/spec/core/binary/index.html) at the
/// given [`Path`].
///
/// See [`parse_module_sections`] and [`Mmap::map`] for more information.
///
/// [`parse_module_sections`]: crate::parse_module_sections
/// [`Mmap::map`]: memmap2::Mmap::map
#[inline]
#[cfg_attr(doc_cfg, doc(cfg(feature = "mmap")))]
pub fn parse_module_sections<P: AsRef<Path>>(path: P) -> parser::Parsed<SectionSequence<Mmap>> {
    parse_module_sections_inner(path.as_ref())
}
