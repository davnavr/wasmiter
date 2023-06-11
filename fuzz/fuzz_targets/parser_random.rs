//! Passes completely random input bytes into [`wasmiter`].

#![no_main]

libfuzzer_sys::fuzz_target!(|wasm: &[u8]| {
    // Normal error conditions are allowed, panics and other crashes are what are tested for
    let _ = wasmiter_fuzz::process_sections(wasm);
    // TODO: Compare against some other reference parser, try printing WAT output too?
});
