(module
    (import "env" "DoTheThings" (func $do_the_things (param i32) (result i32)))

    (tag (param i32))

    (func (param i32) (result i32)
        try (result i32)
            local.get 0
            i32.const 0xFFFF
            i32.add
            call $do_the_things
        catch 0
            drop
            i32.const 0xFFFF0000
        end)
)
