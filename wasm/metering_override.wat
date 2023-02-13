;; this is not a generated WAT
;; there is no WASM equivalent in massa/massa-unit-tests-src

(module
  (memory $memory (export "memory") 1)

  (global $wasmer_metering_remaining_points (export "wasmer_metering_remaining_points") (mut i32) (i32.const 42))

  (func $main (export "main") (result i32)
    (i32.const 42)
    (global.set $wasmer_metering_remaining_points)
    (i32.const 0)
  )
)
