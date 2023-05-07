#![doc = include_str!("../README.md")]
#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(unreachable_pub)]
#![deny(unsafe_op_in_unsafe_fn)]
#![warn(clippy::alloc_instead_of_core)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod allocator;
pub mod component;
pub mod instruction_set;
pub mod parser;

mod sections;

pub use sections::{Section, SectionId, SectionKind, SectionSequence};

pub(crate) use sections::section_id;

use parser::{input, Error, Result, ResultExt as _};

fn parse_module_preamble<I: input::Input>(parser: &mut parser::Decoder<I>) -> Result<()> {
    const MAGIC: [u8; 4] = *b"\0asm";
    const VERSION: [u8; 4] = u32::to_le_bytes(1);

    let mut preamble = [0u8; 8];
    parser
        .bytes_exact(&mut preamble)
        .context("expected WebAssembly module preamble")?;

    if preamble[0..4] != MAGIC {
        return Err(Error::bad_format().with_context("not a valid WebAssembly module"));
    }

    let version = <[u8; 4]>::try_from(&preamble[4..8]).unwrap();
    if version != VERSION {
        let version_number = u32::from_le_bytes(version);
        return Err(parser_bad_format!(
            "unsupported WebAssembly version {version_number} ({version_number:#08X})"
        ));
    }

    Ok(())
}

fn parse_module_binary<I: input::Input, A: allocator::Allocator>(
    binary: I,
    allocator: A,
) -> Result<SectionSequence<I, A>> {
    let mut parser = parser::Decoder::new(binary);
    parse_module_preamble(&mut parser)?;
    Ok(SectionSequence::new_with_allocator(parser, allocator))
}

/// Reads a [WebAssembly module binary](https://webassembly.github.io/spec/core/binary/index.html)
/// with the given [`Allocator`](allocator::Allocator), returning the sequence of sections.
#[inline]
pub fn parse_module_sections_with_allocator<I: input::IntoInput, A: allocator::Allocator>(
    binary: I,
    allocator: A,
) -> Result<SectionSequence<I::In, A>> {
    parse_module_binary(binary.into_input(), allocator)
}

/// Reads a [WebAssembly module binary](https://webassembly.github.io/spec/core/binary/index.html),
/// returning the sequence of sections.
#[inline]
#[cfg(feature = "alloc")]
pub fn parse_module_sections<I: input::IntoInput>(
    binary: I,
) -> Result<SectionSequence<I::In, allocator::Global>> {
    parse_module_binary(binary.into_input(), Default::default())
}

/*
/// Opens a file containing a
/// [WebAssembly module binary](https://webassembly.github.io/spec/core/binary/index.html) at the
/// given [`Path`](std::path::Path).
///
/// See [`parse_module_sections`] for more information.
#[inline]
pub fn parse_module_sections_from_path<P: AsRef<std::path::Path>>(
    path: P,
) -> Result<SectionSequence<parser::FileInput>> {
    parse_module_binary(parser::FileInput::new(std::fs::File::open(path)?))
}
*/
