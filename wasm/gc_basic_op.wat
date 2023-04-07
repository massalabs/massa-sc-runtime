;; this is not a generated WAT
;; there is no WASM equivalent in massa/massa-unit-tests-src

(module
  (type $t0 (func (param i32 i32 i32 i32)))
  (type $t1 (func (param i32) (result i32)))

  (import "env" "abort" (func $env.abort (type $t0)))

  (memory $memory (export "memory") 1)

  (func $__new (export "__new") (param $p0 i32) (param $p1 i32) (result i32)
    (i32.const 0)
  )

  (func $main (export "main") (param $p0 i32) (result i32)
    (local $l0 i32)
    (if
      (i32.gt_u
        (i32.add
          (local.get $p0)
          (i32.const 42))
        (i32.const 84)
      )
      (then
        (call $env.abort
          (i32.const 1)
          (i32.const 2)
          (i32.const 3)
          (i32.const 4))
        (unreachable)
      )
    )
    (local.tee $l0
      (i32.sub
        (i32.const 42)
        (local.get $p0)
      )
    )
    (i32.store offset=0
      (local.get $l0)
    )
    (i32.const 0)
  )
)
