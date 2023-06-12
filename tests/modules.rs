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
fn name_custom_section() {
    let wasm = wat::parse_str(include_str!("modules/name_custom_section.wat")).unwrap();
    insta::assert_debug_snapshot!(wasmiter::parse_module_sections(wasm.as_slice())
        .unwrap()
        .debug_module());
}

macro_rules! check_module_display {
    ($($name:ident,)*) => {$(
        #[test]
        fn $name() {
            const WAT: &str = include_str!(concat!("modules/", stringify!($name), ".wat"));
            let wasm = wat::parse_str(WAT).unwrap();
            let module = wasmiter::parse_module_sections(wasm.as_slice()).unwrap();
            insta::assert_display_snapshot!(module.display_module());
        }
    )*};
}

check_module_display! {
    all_the_things,
    // Case found with libFuzzer
    lots_of_br_table,
    exception_handling,
}
