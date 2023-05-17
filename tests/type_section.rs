use wasmiter::component::{TypesComponent, ValType};

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

    let mut types = TypesComponent::new(0, bytes.as_slice()).unwrap();

    assert_eq!(types.len(), 1);
    assert!(types
        .parse(
            |parameters| {
                assert_eq!(parameters.next().transpose().unwrap(), Some(ValType::I32));
                assert_eq!(parameters.next().transpose().unwrap(), None);
                Ok(())
            },
            |(), results| {
                assert_eq!(results.next().transpose().unwrap(), Some(ValType::I64));
                assert_eq!(results.next().transpose().unwrap(), None);
                Ok(())
            }
        )
        .unwrap()
        .is_some());
}
