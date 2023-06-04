use wasmiter::component::TypesComponent;

#[test]
fn basic_type() {
    let bytes = [
        1u8,  // count
        0x60, // func
        1,    // parameter count
        0x7F, // i32
        1,    // result count
        0x7E, // i64
    ];

    insta::assert_display_snapshot!(TypesComponent::new(0, bytes.as_slice()).unwrap());
}
