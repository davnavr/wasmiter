use wasmiter::component::{TypesComponent, ValType};
use wasmiter::parser::Parser;

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

    let mut types = TypesComponent::new(Parser::new(bytes)).unwrap();

    assert_eq!(types.len(), 1);
    let type_1 = types.next().unwrap().unwrap();
    assert_eq!(type_1.parameter_types(), &[ValType::I32]);
    assert_eq!(type_1.result_types(), &[ValType::I64]);
    assert!(types.next().is_none());
}
