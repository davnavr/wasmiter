(module $my_module_name
  (tag $my_language_eh (param i32 i32))

  (func $mul_six (export "mul_six") (param i32) (result i32)
    local.get 0
    i32.const 6
    i32.mul)

  (func $dance (result i32)
    ;; (local $my_local i32)
    i32.const 42
    call $mul_six))
