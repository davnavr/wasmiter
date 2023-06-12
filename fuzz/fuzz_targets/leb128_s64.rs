#![no_main]

libfuzzer_sys::fuzz_target!(|expected: i64| {
    let mut buffer = [0u8; 10];
    let amount = leb128::write::signed(&mut buffer.as_mut_slice(), expected).unwrap();
    let input: &[u8] = &buffer[0..amount];
    let display = wasmiter::bytes::DebugBytes::from(input);
    match wasmiter::parser::leb128::s64(&mut 0u64, input) {
        Ok(actual) => assert_eq!(
            actual, expected,
            "{actual} should be {expected} {display:?}"
        ),
        Err(e) => panic!("could not parse {expected} {display:?}:\n{e}"),
    }
});
