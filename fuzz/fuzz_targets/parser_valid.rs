#![no_main]

libfuzzer_sys::fuzz_target!(|module: wasmiter_fuzz::ConfiguredModule| {
    let wasm = module.module.to_bytes();
    wasmiter_fuzz::process_sections(wasm.as_slice()).unwrap();
});
