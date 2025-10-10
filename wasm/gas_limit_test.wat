;; Test: infinite loop calling get_current_period until gas runs out
(module
  (import "massa" "assembly_script_get_current_period" (func $get_period (result i64)))
  
  (func (export "test")
    (loop $infinite
      ;; Call get_current_period and drop result
      (call $get_period)
      drop
      
      ;; Continue looping until out of gas
      (br $infinite)
    )
  )
  
  (memory 1)
  (export "memory" (memory 0))
)

