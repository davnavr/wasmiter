#![doc = include_str!("../README.md")]
#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(unreachable_pub)]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![deny(clippy::alloc_instead_of_core)]
#![deny(clippy::std_instead_of_alloc)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod wat;

pub mod bytes;
pub mod component;
pub mod custom;
pub mod index;
pub mod instruction_set;
pub mod parser;
pub mod sections;
pub mod types;

use parser::{Error, Result, ResultExt};

const PREAMBLE_LENGTH: u8 = 8;

fn parse_module_preamble<B: bytes::Bytes>(src: &B) -> Result<()> {
    const MAGIC: [u8; 4] = *b"\0asm";
    const VERSION: [u8; 4] = u32::to_le_bytes(1);

    let mut preamble = [0u8; PREAMBLE_LENGTH as usize];
    parser::bytes_exact(&mut 0, src, &mut preamble)
        .context("expected WebAssembly module preamble")?;

    if preamble[0..4] != MAGIC {
        return Err(Error::bad_format().with_context("not a valid WebAssembly module"));
    }

    let version = <[u8; 4]>::try_from(&preamble[4..8]).unwrap();
    if version != VERSION {
        let version_number = u32::from_le_bytes(version);
        return Err(parser_bad_format_at_offset!(
            "file" @ 0,
            "unsupported WebAssembly version {version_number} ({version_number:#08X})"
        ));
    }

    Ok(())
}

/// Reads a [WebAssembly module binary](https://webassembly.github.io/spec/core/binary/index.html),
/// returning the sequence of sections.
///
/// To interpret the contents of each section, use [`component::KnownSection::interpret`], or in
/// the case of custom sections, [`custom::KnownCustomSection::interpret`].
#[inline]
pub fn parse_module_sections<B: bytes::Bytes>(binary: B) -> Result<sections::SectionSequence<B>> {
    parse_module_preamble(&binary)?;
    Ok(sections::SectionSequence::new(
        u64::from(PREAMBLE_LENGTH),
        binary,
    ))
}

/// Opens a memory-mapped file containing a
/// [WebAssembly module binary](https://webassembly.github.io/spec/core/binary/index.html) at the
/// given [`Path`](std::path::Path).
///
/// See [`parse_module_sections`] and [`Mmap::map`](memmap2::Mmap::map) for more information.
#[inline]
#[cfg(feature = "mmap")]
pub fn parse_module_sections_from_mmap_file<P: AsRef<std::path::Path>>(
    path: P,
) -> Result<sections::SectionSequence<memmap2::Mmap>> {
    let actual_path = path.as_ref();

    let file = std::fs::File::open(actual_path).map_err(|e| {
        crate::parser_bad_input!(
            e.into(),
            "could not open file at path {}",
            actual_path.display()
        )
    })?;

    // Safety: See documentation for Bytes impl on Mmap on how unsafe behavior is "ignored" (sorry)
    let binary = unsafe { memmap2::Mmap::map(&file) };

    parse_module_sections(binary.map_err(|e| {
        crate::parser_bad_input!(
            e.into(),
            "could not create memory map for file at path {}",
            actual_path.display()
        )
    })?)
}
