use wasmiter::parser::Parser;

#[test]
fn examples_u32() {
    macro_rules! assert_eq_decoded {
        ($expected:literal, $actual:expr) => {{
            let mut parser = Parser::new(AsRef::<[u8]>::as_ref($actual));
            assert_eq!($expected, parser.leb128_u32().unwrap());
        }};
    }

    assert_eq_decoded!(0, &[0]);
    assert_eq_decoded!(7, &[7]);
    assert_eq_decoded!(8, &[0x80, 1]);
}
