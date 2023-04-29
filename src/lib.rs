#![doc = include_str!("../README.md")]
#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(unreachable_pub)]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(clippy::undocumented_unsafe_blocks)]

pub mod parser;

mod sections;

pub use sections::{Section, SectionId, SectionKind, SectionSequence};

const MAGIC: [u8; 4] = *b"\0asm";

const VERSION: [u8; 4] = u32::to_le_bytes(1);

fn parse_module_binary<I: parser::Input>(binary: I) -> parser::Result<SectionSequence<I>> {
    use parser::{Error, ResultExt};

    let mut parser = parser::Parser::new(binary);
    let mut preamble = [0u8; 8];
    parser
        .bytes_exact(&mut preamble)
        .context("expected WebAssembly module preamble")?;

    if &preamble[0..4] != &MAGIC {
        return Err(Error::bad_format().with_context("not a valid WebAssembly module"));
    }

    let version = <[u8; 4]>::try_from(&preamble[4..8]).unwrap();
    if version != VERSION {
        let version_number = u32::from_le_bytes(version);
        return Err(Error::bad_format().with_context(format!(
            "unsupported WebAssembly version {version_number} ({version_number:#X})"
        )));
    }

    // TODO: sections sequence
    todo!()
}

/// Reads a [WebAssembly module binary](https://webassembly.github.io/spec/core/binary/index.html),
/// returning the sequence of sections.
#[inline]
pub fn parse_module_sections<I: parser::IntoInput>(
    binary: I,
) -> parser::Result<SectionSequence<I::In>> {
    parse_module_binary(binary.into_input())
}

/// Opens a file containing a
/// [WebAssembly module binary](https://webassembly.github.io/spec/core/binary/index.html) at the
/// given [`Path`](std::path::Path).
///
/// See [`parse_module_sections`] for more information.
#[inline]
pub fn parse_module_sections_from_path<P: AsRef<std::path::Path>>(
    path: P,
) -> parser::Result<SectionSequence<parser::FileInput>> {
    parse_module_binary(parser::FileInput::new(std::fs::File::open(path)?))
}
