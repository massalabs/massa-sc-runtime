use anyhow::{bail, Result};
use execution::run;
use std::env;
use std::fs;
use std::path::Path;

fn _read_files() -> Result<Vec<String>> {
    // TODO: should be or use a read_files(filename: Path) -> String
    let args: Vec<String> = env::args().collect();
    let mut ret = vec![];
    for name in args {
        let path = Path::new(&name);
        if !path.is_file() {
            bail!("{} isn't file", name)
        }
        // TODO: should also handle binary WASM file?!
        if path.extension().unwrap() != "wat" {
            bail!("{} should be in webassembly", name)
        }
        ret.push(fs::read_to_string(path)?);
    }
    Ok(ret)
}

fn main() -> Result<()> {
    let module_wat = r#"
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
       "#;
    run(1, module_wat)
}
