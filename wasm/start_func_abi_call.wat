;; this is not a generated WAT
;; there is no WASM equivalent in massa/massa-unit-tests-src

(module
  (type $t1 (func (param i32)))
  (import "massa" "assembly_script_generate_event" (func $massa.assembly_script_generate_event (type $t1)))

  (memory $memory (export "memory") 1)

  (func $f1
    (call $massa.assembly_script_generate_event (i32.const 42))
  )
  (start $f1)

  (func $main (export "main") (result i32)
    (i32.const 0)
  )
)
