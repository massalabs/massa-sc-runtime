(module
  (memory $0 1)
  (export "memory" (memory $0))
  (data (i32.const 0) "Hi")
  (func (export "main") (result i32)
    i32.const 0  ;; get offset of our data ("Hi")
    i32.const 2  ;; get length of our data
    i32.add))