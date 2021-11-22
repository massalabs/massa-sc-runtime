(module
 (type $i32_i32_=>_i32 (func (param i32 i32) (result i32)))
 (type $none_=>_i32 (func (result i32)))
 (memory $0 0)
 (export "add" (func $assembly/index/add))
 (export "main" (func $assembly/index/main))
 (export "memory" (memory $0))
 (func $assembly/index/add (param $0 i32) (param $1 i32) (result i32)
  local.get $0
  local.get $1
  i32.add
 )
 (func $assembly/index/main (result i32)
  i32.const 12
 )
)
