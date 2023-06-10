(module $my_module_name
  (func $mul_six (export "mul_six") (param i32) (result i32)
    local.get 0
    i32.const 6
    i32.mul))
