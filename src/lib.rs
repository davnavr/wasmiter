#![doc = include_str!("../README.md")]
//! ## Feature Flags
//!
//! - `std`: Enables the usage of [`std`], providing
//!   [`std::error::Error`](https://doc.rust-lang.org/std/error/trait.Error.html) implementations
//!   among other things. Implies the `alloc` flag.
//! - `alloc`: Enables support for heap allocations with [`alloc`]. This allows for more
//!   descriptive [`parser::Error`] messages.
//! - `backtrace`: Enables attaching [`std::backtrace::Backtrace`]s to [`parser::Error`]s. Requires
//!   the `std` flag.
//! - `mmap`: Enables the optional dependency on [`memmap2`](https://docs.rs/memmap2/), which
//!   allows treating a memory mapped file as an [`Input`](input::Input) to the parser. Requires
//!   the `std` flag.
//!
//! [`std`]: https://doc.rust-lang.org/std/
//! [`std::backtrace::Backtrace`]: https://doc.rust-lang.org/std/backtrace/struct.Backtrace.html
//! [`alloc`]: https://doc.rust-lang.org/alloc/

#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(unreachable_pub)]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![deny(clippy::alloc_instead_of_core)]
#![deny(clippy::cast_possible_truncation)]
#![deny(clippy::std_instead_of_alloc)]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(doc_cfg, feature(doc_cfg))]

#[cfg(feature = "alloc")]
extern crate alloc;

mod int;
mod wat;

pub mod component;
pub mod custom;
pub mod index;
pub mod input;
pub mod instruction_set;
pub mod parser;
pub mod sections;
pub mod types;

const _CHECK_POINTER_SIZE: () = if usize::BITS < 32 {
    panic!("wasmiter is not supported in environments with a pointer size less than 32-bits")
};

cfg_if::cfg_if! {
    if #[cfg(feature = "mmap")] {
        mod mmap;

        pub use mmap::parse_module_sections as parse_module_sections_from_mmap_file;
    }
}

const PREAMBLE_LENGTH: u8 = 8;

fn parse_module_preamble<I: input::Input>(input: &I) -> parser::Parsed<()> {
    use parser::ResultExt;

    const MAGIC: [u8; 4] = *b"\0asm";
    const VERSION: [u8; 4] = u32::to_le_bytes(1);

    let mut preamble = [0u8; PREAMBLE_LENGTH as usize];
    parser::bytes_exact(&mut 0, input, &mut preamble)
        .context("expected WebAssembly module preamble")?;

    if preamble[0..4] != MAGIC {
        #[inline(never)]
        #[cold]
        fn bad_magic() -> parser::Error {
            parser::Error::new(parser::ErrorKind::BadWasmMagic)
        }

        return Err(bad_magic());
    }

    let version = <[u8; 4]>::try_from(&preamble[4..8]).unwrap();
    if version != VERSION {
        #[inline(never)]
        #[cold]
        fn unsupported_wasm_version(version: u32) -> parser::Error {
            parser::Error::new(parser::ErrorKind::UnsupportedWasmVersion(version))
                .with_location_context("preamble", 0)
        }

        return Err(unsupported_wasm_version(u32::from_le_bytes(version)));
    }

    Ok(())
}

/// Reads a [WebAssembly module binary](https://webassembly.github.io/spec/core/binary/index.html),
/// returning the sequence of sections.
///
/// To interpret the contents of each section, use [`component::KnownSection::interpret`], or in
/// the case of custom sections, [`custom::KnownCustomSection::interpret`].
#[inline]
pub fn parse_module_sections<I: input::Input>(
    binary: I,
) -> parser::Parsed<sections::SectionSequence<I>> {
    parse_module_preamble(&binary)?;
    Ok(sections::SectionSequence::new(
        u64::from(PREAMBLE_LENGTH),
        binary,
    ))
}
