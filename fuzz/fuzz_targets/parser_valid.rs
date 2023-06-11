#![no_main]

libfuzzer_sys::fuzz_target!(|module: wasmiter_fuzz::ConfiguredModule| {
    let wasm = module.module.to_bytes();
    match wasmiter_fuzz::process_sections(wasm.as_slice()) {
        Ok(()) => (),
        Err(e) => {
            panic!(
                "wasmiter:\n{e}\nwasmprinter:\n{}",
                wasmiter_fuzz::print_reference_wat(&wasm)
            );
        }
    }
});
