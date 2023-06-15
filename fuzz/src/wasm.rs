use libfuzzer_sys::arbitrary::{self, Arbitrary};

#[derive(Clone, Copy, Debug)]
struct RandomConfig {
    allow_start_export: bool,
    allowed_instructions: wasm_smith::InstructionKinds,
    bulk_memory_enabled: bool,
    exceptions_enabled: bool,
    export_everything: bool,
    memory_64_enabled: bool,
    memory_max_size_required: bool,
    min_uleb_size: u8,
    multi_value_enabled: bool,
    reference_types_enabled: bool,
    //relaxed_simd_enabled
    saturating_float_to_int_enabled: bool,
    sign_extension_ops_enabled: bool,
    simd_enabled: bool,
    table_max_size_required: bool,
    tail_call_enabled: bool,
    threads_enabled: bool,
}

impl wasm_smith::Config for RandomConfig {
    fn allow_start_export(&self) -> bool {
        self.allow_start_export
    }

    fn allowed_instructions(&self) -> wasm_smith::InstructionKinds {
        self.allowed_instructions
    }

    fn bulk_memory_enabled(&self) -> bool {
        self.bulk_memory_enabled
    }

    fn exceptions_enabled(&self) -> bool {
        self.exceptions_enabled
    }

    fn export_everything(&self) -> bool {
        self.export_everything
    }

    fn memory64_enabled(&self) -> bool {
        self.memory_64_enabled
    }

    fn memory_max_size_required(&self) -> bool {
        self.memory_max_size_required
    }

    fn min_uleb_size(&self) -> u8 {
        self.min_uleb_size
    }

    fn multi_value_enabled(&self) -> bool {
        self.multi_value_enabled
    }

    fn reference_types_enabled(&self) -> bool {
        self.reference_types_enabled
    }

    fn saturating_float_to_int_enabled(&self) -> bool {
        self.saturating_float_to_int_enabled
    }

    fn sign_extension_ops_enabled(&self) -> bool {
        self.sign_extension_ops_enabled
    }

    fn simd_enabled(&self) -> bool {
        self.simd_enabled
    }

    fn table_max_size_required(&self) -> bool {
        self.table_max_size_required
    }

    fn tail_call_enabled(&self) -> bool {
        self.tail_call_enabled
    }

    fn threads_enabled(&self) -> bool {
        self.threads_enabled
    }
}

/// A pseudorandomly generated WebAssembly binary.
pub struct Wasm {
    module: wasm_smith::Module,
}

impl Wasm {
    pub fn into_bytes(self) -> Vec<u8> {
        self.module.to_bytes()
    }
}

impl std::fmt::Debug for Wasm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.module, f)
    }
}

impl Arbitrary<'_> for RandomConfig {
    fn arbitrary(u: &mut arbitrary::Unstructured<'_>) -> arbitrary::Result<Self> {
        Ok(Self {
            allow_start_export: u.arbitrary()?,
            allowed_instructions: {
                use wasm_smith::InstructionKind;

                const LEN: usize = 8;

                let mut kinds: [InstructionKind; LEN] = [
                    InstructionKind::Control,
                    InstructionKind::Memory,
                    InstructionKind::Numeric,
                    InstructionKind::Parametric,
                    InstructionKind::Reference,
                    InstructionKind::Table,
                    InstructionKind::Variable,
                    InstructionKind::Vector,
                ];

                kinds.rotate_right(u.choose_index(LEN)?);
                wasm_smith::InstructionKinds::new(&kinds[..u.choose_index(LEN)?])
            },
            bulk_memory_enabled: u.arbitrary()?,
            exceptions_enabled: u.arbitrary()?,
            export_everything: u.arbitrary()?,
            memory_64_enabled: u.arbitrary()?,
            memory_max_size_required: u.arbitrary()?,
            min_uleb_size: u.int_in_range(1u8..=5u8)?,
            multi_value_enabled: u.arbitrary()?,
            reference_types_enabled: u.arbitrary()?,
            saturating_float_to_int_enabled: u.arbitrary()?,
            sign_extension_ops_enabled: u.arbitrary()?,
            simd_enabled: u.arbitrary()?,
            table_max_size_required: u.arbitrary()?,
            tail_call_enabled: u.arbitrary()?,
            threads_enabled: u.arbitrary()?,
        })
    }
}

impl Arbitrary<'_> for Wasm {
    fn arbitrary(u: &mut arbitrary::Unstructured<'_>) -> arbitrary::Result<Self> {
        Ok(Self {
            module: u
                .arbitrary::<wasm_smith::ConfiguredModule<RandomConfig>>()?
                .module,
        })
    }
}
