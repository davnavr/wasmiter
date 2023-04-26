#![doc = include_str!("../README.md")]
#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(unreachable_pub)]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(clippy::undocumented_unsafe_blocks)]

pub mod parser;

/// Parses the sections of an in-memory WebAssembly module.
pub fn parse_sections_from_bytes(bytes: &[u8]) -> parser::Result<parser::SectionsParser> {
    parser::SectionsParser::from_input(std::io::Cursor::new(bytes))
}

/// Parses the sections of a WebAssembly module file.
pub fn parse_sections_from_path<P: AsRef<std::path::Path>>(
    path: P,
) -> parser::Result<parser::SectionsParser> {
    parser::SectionsParser::from_input(parser::FileInput::from_path(path)?)
}
