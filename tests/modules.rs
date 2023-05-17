use wasmiter::component::{self, KnownSection, ValType};
use wasmiter::instruction_set::Instruction;

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
        if let Ok(known_section) =
            KnownSection::try_from_with_allocator(section.unwrap(), wasmiter::allocator::Global)
        {
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

    let mut exports = exports.unwrap();
    let ex = exports.parse().unwrap().unwrap();
    assert_eq!("add_five", ex.name());
    assert!(
        matches!(ex.kind(), component::ExportKind::Function(i) if i.to_u32() == 0),
        "incorrect kind {:?}",
        ex.kind()
    );
    assert!(exports.parse().unwrap().is_none());

    let mut code = code.unwrap();
    let entry = code.parse().unwrap().unwrap();
    entry
        .read(
            |locals| {
                assert_eq!(locals.count(), 0);
                Ok(())
            },
            |(), instrs| {
                instrs
                    .next(|i| {
                        assert!(
                            matches!(i, Instruction::LocalGet(l) if l.to_u32() == 0),
                            "{i:?}"
                        );
                        Ok(())
                    })
                    .unwrap()
                    .unwrap();
                instrs
                    .next(|i| {
                        assert!(matches!(i, Instruction::I32Const(5)), "{i:?}");
                        Ok(())
                    })
                    .unwrap()
                    .unwrap();
                instrs
                    .next(|i| {
                        assert!(matches!(i, Instruction::I32Add), "{i:?}");
                        Ok(())
                    })
                    .unwrap()
                    .unwrap();
                instrs
                    .next(|i| {
                        assert!(matches!(i, Instruction::End), "{i:?}");
                        Ok(())
                    })
                    .unwrap()
                    .unwrap();
                assert!(instrs.next(|_| Ok(())).is_none());
                Ok(())
            },
        )
        .unwrap();
    assert!(code.parse().unwrap().is_none());
}
