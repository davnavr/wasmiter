#![no_main]

libfuzzer_sys::fuzz_target!(|expected: i32| {
    let mut buffer = [0u8; 5];
    let amount = leb128::write::signed(&mut buffer.as_mut_slice(), i64::from(expected)).unwrap();
    let input: &[u8] = &buffer[0..amount];
    let display = wasmiter::bytes::DebugBytes::from(input);
    let mut offset = 0u64;
    match wasmiter::parser::leb128::s32(&mut offset, input) {
        Ok(actual) => {
            assert_eq!(
                actual, expected,
                "{actual} should be {expected} {display:?}"
            );
            assert_eq!(offset, amount as u64, "byte lengths differ for {display:?}");
        }
        Err(e) => panic!("could not parse {expected} {display:?}:\n{e}"),
    }
});
