#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|module: wasmiter_fuzz::ConfiguredModule| {
    // fuzzed code goes here
});
