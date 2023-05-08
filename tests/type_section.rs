use wasmiter::component::{TypesComponent, ValType};
use wasmiter::parser::Decoder;

#[test]
fn basic_type() {
    let bytes = &[
        1u8,  // count
        0x60, // func
        1,    // parameter count
        0x7F, // i32
        1,    // result count
        0x7E, // i64
    ];

    let mut types = TypesComponent::new(Decoder::new(bytes)).unwrap();

    assert_eq!(TypesComponent::count(&types), 1);
    let type_1 = Iterator::next(&mut types).unwrap().unwrap();
    assert_eq!(type_1.0, &[ValType::I32]);
    assert_eq!(type_1.1, &[ValType::I64]);
    assert!(Iterator::next(&mut types).is_none());
}
