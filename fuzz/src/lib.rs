//! Helper structs and functions for fuzzing targets.

use wasmiter::{component::KnownSection, custom::KnownCustomSection};

mod config;

pub use config::{ConfiguredModule, WasmiterConfig};

pub fn process_sections(wasm: &[u8]) -> wasmiter::parser::Result<()> {
    for result in wasmiter::parse_module_sections(wasm)? {
        match KnownSection::interpret(result?) {
            Ok(known) => match known? {
                KnownSection::Type(mut types) => loop {
                    let result = types.parse(
                        |params| {
                            for item in params {
                                let _ = item?;
                            }
                            Ok(())
                        },
                        |(), results| {
                            for item in results {
                                let _ = item?;
                            }
                            Ok(())
                        },
                    )?;

                    if result.is_none() {
                        break;
                    }
                },
                KnownSection::Import(imports) => {
                    for result in imports {
                        let entry = result?;
                        let _ = entry.module().try_into_string()?;
                        let _ = entry.name().try_into_string()?;
                    }
                }
                KnownSection::Function(functions) => {
                    for result in functions {
                        let _ = result?;
                    }
                }
                KnownSection::Table(tables) => {
                    for result in tables {
                        let _ = result?;
                    }
                }
                KnownSection::Memory(mems) => {
                    for result in mems {
                        let _ = result?;
                    }
                }
                KnownSection::Global(mut globals) => loop {
                    if globals.parse(|_, _| Ok(()))?.is_none() {
                        break;
                    }
                },
                KnownSection::Export(exports) => {
                    for result in exports {
                        let _ = result?.name().try_into_string()?;
                    }
                }
                KnownSection::Start(_) => (),
                KnownSection::Element(mut elements) => loop {
                    if elements.parse(|_| Ok(()), |(), _| Ok(()))?.is_none() {
                        break;
                    }
                },
                KnownSection::Code(code) => {
                    for result in code {
                        let entry = result?;
                        entry.read(|_| wasmiter::parser::Result::Ok(()), |(), _| Ok(()))?;
                    }
                }
                KnownSection::Data(mut data) => loop {
                    if data.parse(|_| Ok(()), |(), _| Ok(()))?.is_none() {
                        break;
                    }
                },
                KnownSection::DataCount(_) => (),
                KnownSection::Tag(tag) => {
                    for result in tag {
                        let _ = result?;
                    }
                }
                bad => panic!("unsupported section {:?}", bad.id()),
            },
            Err(possibly_custom) => {
                if let Ok(result) =
                    wasmiter::custom::CustomSection::try_from_section(possibly_custom)
                {
                    if let Ok(known) = KnownCustomSection::interpret(result?) {
                        match known {
                            KnownCustomSection::Name(name) => {
                                for result in name {
                                    use wasmiter::custom::name::NameSubsection;

                                    if let Ok(result) = result {
                                        match result? {
                                            NameSubsection::ModuleName(name) => {
                                                let _ = name.try_into_string()?;
                                            }
                                            NameSubsection::FunctionName(name_map) => {
                                                for result in name_map {
                                                    let name_assoc = result?;
                                                    let _ = name_assoc.name().try_into_string()?;
                                                }
                                            }
                                            NameSubsection::LocalName(mut indirect_name_map) => {
                                                loop {
                                                    let result = indirect_name_map.parse(
                                                        |_, name_map| {
                                                            for result in name_map {
                                                                let name_assoc = result?;
                                                                let _ = name_assoc
                                                                    .name()
                                                                    .try_into_string()?;
                                                            }
                                                            Ok(())
                                                        },
                                                    )?;

                                                    if result.is_some() {
                                                        break;
                                                    }
                                                }
                                            }
                                            NameSubsection::TagName(name_map) => {
                                                for result in name_map {
                                                    let name_assoc = result?;
                                                    let _ = name_assoc.name().try_into_string()?;
                                                }
                                            }
                                            bad => {
                                                panic!("unsupported name subsection {:?}", bad.id())
                                            }
                                        }
                                    }
                                }
                            }
                            bad => panic!("unsupported custom section {:?}", bad.name()),
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
