use wasmiter::component;

#[test]
fn type_section() {
    let bytes = [
        1u8,  // count
        0x60, // func
        1,    // parameter count
        0x7F, // i32
        1,    // result count
        0x7E, // i64
    ];

    insta::assert_display_snapshot!(component::TypesComponent::new(0, bytes.as_slice()).unwrap());
}

#[test]
fn export_section() {
    let bytes = [
        1u8, // count
        0xC, // name length
        0x6D, 0x79, 0x45, 0x78, 0x70, 0x6f, 0x72, 0x74, 0x4e, 0x61, 0x6d, 0x65,
        0, // export func
        0, // func idx
    ];

    insta::assert_display_snapshot!(component::ExportsComponent::new(0, bytes.as_slice()).unwrap());
}
