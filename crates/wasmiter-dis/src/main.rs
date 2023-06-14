//! Uses [`wasmiter`] to convert a `.wasm` file into WebAssembly Text (`.wat`).

#![deny(unreachable_pub)]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(clippy::undocumented_unsafe_blocks)]

use clap::Parser;
use std::io::Write;

// TODO: How compatible with [`wasm2wat`] should this be?
// [`wasm2wat`]: https://webassembly.github.io/wabt/doc/wasm2wat.1.html

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// The WebAssembly binary `.wasm` file to read
    file: std::path::PathBuf,
    /// Where to write the generated WebAssembly Text, defaults to stdout
    #[arg(short, long)]
    output: Option<std::path::PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    cfg_if::cfg_if! {
        if #[cfg(any(unix, windows))] {
            let sections = wasmiter::parse_module_sections_from_mmap_file(cli.file)?;
        } else {
            compile_error!("std::io::File fallback is not yet implemented")
        }
    };

    let mut file;
    let mut stdout;

    let output: &mut dyn Write = if let Some(destination) = cli.output.as_ref() {
        file = std::fs::File::create(destination)?;
        &mut file
    } else {
        stdout = std::io::stdout().lock();
        &mut stdout
    };

    let mut buffered = std::io::BufWriter::new(output);
    writeln!(&mut buffered, "{}", sections.display_module())?;
    buffered.flush()?;

    Ok(())
}
