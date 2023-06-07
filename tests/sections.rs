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
        0x6D, 0x79, 0x45, 0x78, 0x70, 0x6f, 0x72, 0x74, 0x4E, 0x61, 0x6D, 0x65,
        0, // export func
        0, // funcidx
    ];

    insta::assert_display_snapshot!(component::ExportsComponent::new(0, bytes.as_slice()).unwrap());
}

#[test]
fn import_section() {
    let bytes = [
        4u8, // count

        // Import 0
        3, // name length
        0x65, 0x6E, 0x76,
        0xB, // name length
        0x64, 0x6F, 0x53, 0x6F, 0x6D, 0x65, 0x53, 0x74, 0x75, 0x66, 0x66,
        0, // import func
        0, // typeidx

        // Import 1
        3, // name length
        0x65, 0x6E, 0x76,
        6, // name length
        0x6D, 0x65, 0x6D, 0x6F, 0x72, 0x79,
        2, // import memory
        0, // limit w/o maximum
        0x10, // limit minimum

        // Import 2
        2, // name length
        0x72, 0x74,
        0xA, // name length
        0x72, 0x65, 0x66, 0x65, 0x72, 0x65, 0x6E, 0x63, 0x65, 0x73,
        1, // import table,
        0x6F, // externref
        0, 0, // limits

        // Import 3
        2, // name length
        0x72, 0x74,
        8, // name length
        0x73, 0x74, 0x61, 0x63, 0x6B, 0x70, 0x74, 0x72,
        3, // import global
        0x7F, // i32
        1, // mutable
    ];

    insta::assert_display_snapshot!(component::ImportsComponent::new(0, bytes.as_slice()).unwrap());
}
