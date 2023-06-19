(module
    (import "sample_api" "_handles" (table $_HANDLES 0 externref))
    (import "sample_api" "_handles_start" (global (mut i32)))
    (import "sample_api" "panic" (tag (param i32)))

    (import "extern" "memory" (memory 16))
    (import "extern" "fun_startup" (func))
    (import "extern" "fun_link_startup" (func $fun_startup (result i32)))

    (table $FUNCS 0 funcref)
    (global $FUNCS_PTR (mut i32) i32.const 0)

    (memory $SCRATCH 0 16)
    (global $SCRATCH_PTR (mut i32) i32.const 0)

    (data "el amor")

    ;; Thanks libFuzzer :)
    (data "\FF\FF\00\51\51\51\51\51\51\51\51\51\51\51\51\51\51\07")

    (func $main (param i32 i32) (result i32)
        i32.const 0)

    (func $_start
        ;; Thanks libFuzzer
        i32.const -669246436
        drop
        nop)

    (func $_init
      	call $fun_startup
        drop)

    (func (result i32)
        (local i32)
        (local i32)
        (local i32)
        (local i32)
        local.get 0)

    (export "_start" (func $_start))

    (start $_init)
)
