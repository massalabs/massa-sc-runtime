;; Test: infinite loop with pure WASM instructions (arithmetic, branching) until gas runs out
(module
  (func (export "test")
    (local i32 i32 i32)
    (local.set 0 (i32.const 1))
    (local.set 1 (i32.const 2))
    
    (loop $infinite
      ;; Arithmetic operations
      (local.set 2 (i32.add (local.get 0) (local.get 1)))
      (local.set 2 (i32.sub (local.get 2) (i32.const 1)))
      (local.set 2 (i32.mul (local.get 2) (i32.const 2)))
      (local.set 2 (i32.div_u (local.get 2) (i32.const 2)))
      
      ;; Comparisons and branching
      (local.get 2)
      (i32.const 1000)
      (i32.lt_u)
      (if
        (then
          (local.set 0 (i32.add (local.get 0) (i32.const 1)))
        )
        (else
          (local.set 1 (i32.add (local.get 1) (i32.const 1)))
        )
      )
      
      ;; More arithmetic
      (local.set 2 (i32.rem_u (local.get 0) (i32.const 7)))
      (local.set 2 (i32.and (local.get 2) (i32.const 15)))
      (local.set 2 (i32.or (local.get 2) (i32.const 3)))
      (local.set 2 (i32.xor (local.get 2) (i32.const 5)))
      
      ;; Continue looping until out of gas
      (br $infinite)
    )
  )
  
  (memory 1)
  (export "memory" (memory 0))
)

