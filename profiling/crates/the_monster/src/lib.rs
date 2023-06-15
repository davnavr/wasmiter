//! The goal here is to make a large WASM file such that a profiler will see which wasmiter
//! functions are slow.

#![allow(clippy::missing_safety_doc)]

type Module = wasmiter::sections::SectionSequence<&'static [u8]>;

#[repr(transparent)]
pub struct ModuleSections(*mut Module);

pub unsafe extern "C" fn wasmiter_create_module_sections(
    len: usize,
    ptr: *const u8,
) -> ModuleSections {
    let slice = unsafe { std::slice::from_raw_parts::<'static, u8>(ptr, len) };

    ModuleSections(Box::leak(Box::new(wasmiter::parse_module_sections(slice).unwrap())) as *mut _)
}

pub unsafe extern "C" fn wasmiter_free_module_sections(module: ModuleSections) {
    std::mem::drop(Box::from_raw(module.0));
}

pub unsafe extern "C" fn wasmiter_print_module(module: ModuleSections) {
    let module: &_ = unsafe { &*module.0 };
    println!("{}", module.display_module());
}

pub unsafe extern "C" fn wasmiter_print_module_debug(module: ModuleSections) {
    let module: &_ = unsafe { &*module.0 };
    println!("{:?}", module.debug_module());
}

#[repr(transparent)]
pub struct Buffer(*mut Vec<u8>);

pub unsafe extern "C" fn wasmiter_reassemble_module(module: ModuleSections) -> Buffer {
    let module: &_ = unsafe { &*module.0 };
    let text = format!("{}", module.display_module());
    Buffer(Box::leak(Box::new(wat::parse_str(text).unwrap())) as *mut _)
}

pub unsafe extern "C" fn wasmiter_free_buffer(buffer: Buffer) {
    std::mem::drop(Box::from_raw(buffer.0));
}
