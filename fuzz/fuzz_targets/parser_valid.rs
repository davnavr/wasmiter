#![no_main]

use libfuzzer_sys::fuzz_target;

fn process_sections(wasm: &[u8]) -> wasmiter::parser::Result<()> {
    for result in wasmiter::parse_module_sections(wasm).unwrap() {
        match wasmiter::component::KnownSection::interpret(result)? {
            Ok(known) => {
                //known.finish()
                let _ = known?;
            }
            Err(possibly_custom) => if let Ok(result) = wasmiter::custom::CustomSection::try_from_section(possibly_custom) {
                if let Ok(known) = result? {
                    let _ = known;
                }
            },
        }
    }
    Ok(())
}

fuzz_target!(|module: wasm_smith::Module| {
    let wasm = module.to_bytes();
    process_sections(wasm.as_slice()).unwrap();
});
