#![no_main]

libfuzzer_sys::fuzz_target!(|expected: i64| {
    let mut buffer = [0u8; 10];
    let amount = leb128::write::signed(&mut buffer.as_mut_slice(), expected).unwrap();
    let input: &[u8] = &buffer[0..amount];
    let display = wasmiter::input::HexDump::from(input);
    let mut offset = 0u64;
    match wasmiter::parser::leb128::s64(&mut offset, input) {
        Ok(actual) => {
            assert_eq!(
                actual, expected,
                "{actual} should be {expected}\n{display:#}"
            );
            assert_eq!(offset, amount as u64, "byte lengths differ\n{display:#}");
        }
        Err(e) => panic!("could not parse {expected}:\n{e}\n{display:#}"),
    }
});
