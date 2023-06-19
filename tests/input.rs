use wasmiter::input::{HexDump, Window};

const DATA: &[u8] =
    b"and the man went out for jury duty and exclaimed, \"Segmentation fault (core dumped)\"\x00";

#[test]
fn hex_dump_debug() {
    insta::assert_debug_snapshot!(HexDump::from(DATA));
}

#[test]
fn hex_dump_display() {
    insta::assert_display_snapshot!(format_args!("{:#}", HexDump::from(DATA)));
}

#[test]
fn hex_dump_display_at_weird_offset() {
    let window = Window::with_offset_and_length(DATA, 3, 45);
    insta::assert_display_snapshot!(format_args!("{:#}", HexDump::from(window)));
}
