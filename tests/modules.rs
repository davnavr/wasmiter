#[test]
fn basic_module() {
    let wat = r#"(module
    (func (export "add_five") (param i32) (result i32)
        local.get 0
        i32.const 5
        i32.add))
"#;
    let wasm = wat::parse_str(wat).unwrap();
    insta::assert_display_snapshot!(wasmiter::parse_module_sections(wasm.as_slice())
        .unwrap()
        .display_module());
}

#[test]
fn exception_handling() {
    let wasm = wat::parse_str(include_str!("modules/exception_handling.wat")).unwrap();
    insta::assert_display_snapshot!(wasmiter::parse_module_sections(wasm.as_slice())
        .unwrap()
        .display_module());
}

#[test]
fn all_the_things() {
    let wasm = wat::parse_str(include_str!("modules/all_the_things.wat")).unwrap();
    insta::assert_display_snapshot!(wasmiter::parse_module_sections(wasm.as_slice())
        .unwrap()
        .display_module());
}

#[test]
fn name_custom_section() {
    let wasm = wat::parse_str(include_str!("modules/name_custom_section.wat")).unwrap();
    insta::assert_debug_snapshot!(wasmiter::parse_module_sections(wasm.as_slice())
        .unwrap()
        .debug_module());
}

#[test]
fn lots_of_br_table() {
    // Case found with libFuzzer
    let wasm = wat::parse_str(include_str!("modules/lots_of_br_table.wat")).unwrap();
    insta::assert_display_snapshot!(wasmiter::parse_module_sections(wasm.as_slice())
        .unwrap()
        .display_module());
}
