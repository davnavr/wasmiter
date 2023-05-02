use wasmiter::parser::Parser;

#[test]
fn examples_u32() {
    macro_rules! assert_eq_decoded {
        ($expected:expr, $actual:expr) => {{
            let mut parser = Parser::new(AsRef::<[u8]>::as_ref($actual));
            assert_eq!($expected, parser.leb128_u32().unwrap());
        }};
    }

    assert_eq_decoded!(0, &[0]);
    assert_eq_decoded!(0x7F, &[0x7F]);
    assert_eq_decoded!(0x80, &[0x80, 1]);
    assert_eq_decoded!(3, &[0x83, 0]); // Trailing zeroes are allowed, used in linker output
    assert_eq_decoded!(0x3FFF, &[0xFF, 0x7F]);
    assert_eq_decoded!(0x4000, &[0x80, 0x80, 1]);
    assert_eq_decoded!(0x1FFFFF, &[0xFF, 0xFF, 0x7F]);
    assert_eq_decoded!(0x200000, &[0x80, 0x80, 0x80, 1]);
    assert_eq_decoded!(15, &[0x8F, 0x80, 0x80, 0]);
    assert_eq_decoded!(0x0FFFFFFF, &[0xFF, 0xFF, 0xFF, 0x7F]);
    assert_eq_decoded!(0x10000000, &[0x80, 0x80, 0x80, 0x80, 1]);
    assert_eq_decoded!(u32::MAX, &[0xFF, 0xFF, 0xFF, 0xFF, 0x0F]);
}
