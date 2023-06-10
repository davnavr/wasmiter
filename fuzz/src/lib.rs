use libfuzzer_sys::{
    arbitrary::{self, Arbitrary},
};

#[derive(Arbitrary, Clone, Copy, Debug, Default)]
pub struct WasmiterConfig;

impl wasm_smith::Config for WasmiterConfig {
    fn bulk_memory_enabled(&self) -> bool {
        true
    }

    fn reference_types_enabled(&self) -> bool {
        true
    }

    fn tail_call_enabled(&self) -> bool {
        true
    }

    fn simd_enabled(&self) -> bool {
        true
    }

    fn exceptions_enabled(&self) -> bool {
        true
    }

    fn multi_value_enabled(&self) -> bool {
        true
    }

    fn saturating_float_to_int_enabled(&self) -> bool {
        true
    }

    fn sign_extension_ops_enabled(&self) -> bool {
        true
    }

    fn memory64_enabled(&self) -> bool {
        true
    }

    fn generate_custom_sections(&self) -> bool {
        true
    }

    fn threads_enabled(&self) -> bool {
        true
    }
}

pub type ConfiguredModule = wasm_smith::ConfiguredModule<WasmiterConfig>;
