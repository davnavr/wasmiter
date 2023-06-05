use wasmiter::parser::name::Name;

#[test]
fn basic_type() {
    let inputs = [
        "Hello World!".as_bytes(),
        &[0x61, 0x62, 0x63],
        &[0xFF, 0xFF],
        &[0],
        "espa\u{F1}ol".as_bytes(),
        "\u{420}\u{43e}\u{441}\u{441}\u{438}\u{44f}".as_bytes(),
        "Fuerza A\u{E9}rea Mexicana".as_bytes(),
        "\u{1f643}".as_bytes(),
    ];

    insta::assert_debug_snapshot!(inputs
        .iter()
        .map(|bytes| Name::try_from(*bytes))
        .collect::<Box<[_]>>());
}
