#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|module: wasmiter_fuzz::ConfiguredModule| {
    let wasm = module.module.to_bytes();
    let sections = wasmiter::parse_module_sections(wasm.as_slice()).unwrap();
    let module = sections.display_module();
    let text = format!("{module}");
    match wat::parse_str(&text) {
        Ok(_) => (),
        Err(e) => {
            panic!(
                "wat:\n{e}\nwasmiter:\n{text}\nwasmprinter:\n{}",
                wasmiter_fuzz::print_reference_wat(&wasm)
            )
        }
    }
});
