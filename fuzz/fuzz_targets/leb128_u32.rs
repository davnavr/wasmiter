#![no_main]

libfuzzer_sys::fuzz_target!(|expected: u32| {
    let mut buffer = [0u8; 5];
    let amount = leb128::write::unsigned(&mut buffer.as_mut_slice(), u64::from(expected)).unwrap();
    let input: &[u8] = &buffer[0..amount];
    let display = wasmiter::bytes::DebugBytes::from(input);
    match wasmiter::parser::leb128::u32(&mut 0u64, input) {
        Ok(actual) => assert_eq!(
            actual, expected,
            "{actual} should be {expected} {display:?}"
        ),
        Err(e) => panic!("could not parse {expected} {display:?}:\n{e}"),
    }
});
