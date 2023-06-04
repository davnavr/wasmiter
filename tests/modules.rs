use wasmiter::{
    component::{self, KnownSection},
    instruction_set::Instruction,
    types::ValType,
};

#[test]
fn basic_module() {
    let wat = r#"(module
    (func (export "add_five") (param i32) (result i32)
        local.get 0
        i32.const 5
        i32.add))
"#;
    let wasm = wat::parse_str(wat).unwrap();
    let module = wasmiter::parse_module_sections(wasm.as_slice()).unwrap();

    let mut types = None;
    let mut exports = None;
    let mut code = None;

    for section in module {
        if let Ok(known_section) = KnownSection::try_from_section(section.unwrap()) {
            match known_section.unwrap() {
                KnownSection::Type(ty) => types = Some(ty),
                KnownSection::Export(ex) => exports = Some(ex),
                KnownSection::Code(c) => code = Some(c),
                _ => (),
            }
        }
    }

    let mut types = types.unwrap();
    assert!(types
        .parse(
            |params| {
                assert_eq!(Some(ValType::I32), params.next().transpose().unwrap());
                assert!(params.is_empty());
                Ok(())
            },
            |(), result_types| {
                assert_eq!(Some(ValType::I32), result_types.next().transpose().unwrap());
                assert!(result_types.is_empty());
                Ok(())
            }
        )
        .unwrap()
        .is_some());
    assert!(types.is_empty());

    let mut name_buffer = std::vec::Vec::with_capacity(8);
    let mut exports = exports.unwrap();
    let ex = exports
        .parse_with_buffer(&mut name_buffer)
        .unwrap()
        .unwrap();
    assert_eq!("add_five", ex.name());
    assert!(
        matches!(ex.kind(), component::ExportKind::Function(i) if i.to_u32() == 0),
        "incorrect kind {:?}",
        ex.kind()
    );
    assert!(exports
        .parse_with_buffer(&mut name_buffer)
        .unwrap()
        .is_none());

    let mut code = code.unwrap();
    let entry = code.parse().unwrap().unwrap();
    entry
        .read(
            |locals| {
                assert_eq!(locals.count(), 0);
                wasmiter::parser::Result::Ok(())
            },
            |(), instrs| {
                instrs
                    .next(|i| {
                        assert!(
                            matches!(i, Instruction::LocalGet(l) if l.to_u32() == 0),
                            "{i:?}"
                        );
                        wasmiter::parser::Result::Ok(())
                    })
                    .unwrap()
                    .unwrap();
                instrs
                    .next(|i| {
                        assert!(matches!(i, Instruction::I32Const(5)), "{i:?}");
                        wasmiter::parser::Result::Ok(())
                    })
                    .unwrap()
                    .unwrap();
                instrs
                    .next(|i| {
                        assert!(matches!(i, Instruction::I32Add), "{i:?}");
                        wasmiter::parser::Result::Ok(())
                    })
                    .unwrap()
                    .unwrap();
                instrs
                    .next(|i| {
                        assert!(matches!(i, Instruction::End), "{i:?}");
                        wasmiter::parser::Result::Ok(())
                    })
                    .unwrap()
                    .unwrap();
                assert!(instrs.next(|_| wasmiter::parser::Result::Ok(())).is_none());
                Ok(())
            },
        )
        .unwrap();
    assert!(code.parse().unwrap().is_none());
}
