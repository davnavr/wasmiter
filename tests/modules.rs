#[test]
fn basic_module() {
    let wat = r#"(module
    (func (export "add_five") (param i32) (result i32)
        local.get 0
        i32.const 5
        i32.add))
"#;
    let wasm = wat::parse_str(wat).unwrap();
    insta::assert_display_snapshot!(&wasmiter::parse_module_sections(wasm.as_slice()).unwrap());
}

#[test]
fn exception_handling() {
    let wasm = wat::parse_str(include_str!("modules/exception_handling.wat")).unwrap();
    insta::assert_display_snapshot!(&wasmiter::parse_module_sections(wasm.as_slice()).unwrap());
}

#[test]
fn all_the_things() {
    let wasm = wat::parse_str(include_str!("modules/all_the_things.wat")).unwrap();
    insta::assert_display_snapshot!(&wasmiter::parse_module_sections(wasm.as_slice()).unwrap());
}
