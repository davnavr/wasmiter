#[test]
fn basic_module() {
    let wat = r#"(module
    (func (export "add_five") (param i32) (result i32)
        local.get 0
        i32.const 5
        i32.add))
"#;
    let wasm = wat::parse_str(wat).unwrap();
    insta::assert_debug_snapshot!(wasmiter::parse_module_sections(wasm.as_slice()).unwrap());
}
