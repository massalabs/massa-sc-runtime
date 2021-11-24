(module
 (type $i32_=>_i32 (func (param i32) (result i32)))
 (type $i32_i32_=>_i32 (func (param i32 i32) (result i32)))
 (type $i32_i32_i32_i32_=>_none (func (param i32 i32 i32 i32)))
 (type $none_=>_i32 (func (result i32)))
 (import "index" "call" (func $assembly/index/call (param i32) (result i32)))
 (import "env" "abort" (func $~lib/builtins/abort (param i32 i32 i32 i32)))
 (global $~lib/memory/__stack_pointer (mut i32) (i32.const 17468))
 (memory $0 1)
 (data (i32.const 1036) ",")
 (data (i32.const 1048) "\01\00\00\00\16\00\00\00h\00e\00l\00l\00o\00 \00w\00o\00r\00l\00d")
 (export "add" (func $assembly/index/add))
 (export "main" (func $assembly/index/main))
 (export "memory" (memory $0))
 (export "call" (func $export:assembly/index/call))
 (func $assembly/index/add (param $0 i32) (param $1 i32) (result i32)
  local.get $0
  local.get $1
  i32.add
 )
 (func $assembly/index/main (result i32)
  (local $0 i32)
  global.get $~lib/memory/__stack_pointer
  i32.const 4
  i32.sub
  global.set $~lib/memory/__stack_pointer
  global.get $~lib/memory/__stack_pointer
  i32.const 1084
  i32.lt_s
  if
   i32.const 17488
   i32.const 17536
   i32.const 1
   i32.const 1
   call $~lib/builtins/abort
   unreachable
  end
  global.get $~lib/memory/__stack_pointer
  local.tee $0
  i32.const 0
  i32.store
  local.get $0
  i32.const 1056
  i32.store
  i32.const 1056
  call $assembly/index/call
  drop
  i32.const 1052
  i32.load
  i32.const 1
  i32.shr_u
  global.get $~lib/memory/__stack_pointer
  i32.const 4
  i32.add
  global.set $~lib/memory/__stack_pointer
 )
 (func $export:assembly/index/call (param $0 i32) (result i32)
  global.get $~lib/memory/__stack_pointer
  i32.const 4
  i32.sub
  global.set $~lib/memory/__stack_pointer
  global.get $~lib/memory/__stack_pointer
  i32.const 1084
  i32.lt_s
  if
   i32.const 17488
   i32.const 17536
   i32.const 1
   i32.const 1
   call $~lib/builtins/abort
   unreachable
  end
  global.get $~lib/memory/__stack_pointer
  local.get $0
  i32.store
  local.get $0
  call $assembly/index/call
  global.get $~lib/memory/__stack_pointer
  i32.const 4
  i32.add
  global.set $~lib/memory/__stack_pointer
 )
)
