use execution::run;
use std::env;
use std::fs;
use std::io::{self, ErrorKind};
use std::path::Path;

fn _read_files() -> Result<Vec<String>, io::Error> {
    let args: Vec<String> = env::args().collect();
    let mut ret = vec![];
    for name in args {
        let path = Path::new(&name);
        if !path.is_file() {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                format!("{} isn't file", name),
            ));
        }
        if path.extension().unwrap() != "wat" {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                format!("{} should be in webassembly", name),
            ));
        }
        ret.push(fs::read_to_string(path)?);
    }
    Ok(ret)
}

fn main() -> anyhow::Result<(), Box<dyn std::error::Error>> {
    let module_wat = r#"
    ;; TODO: put some Python interpreter here!
    (module
    (type $t0 (func (param i32) (result i32)))
    (func $add_one (export "add_one") (type $t0) (param $p0 i32) (result i32)
        get_local $p0
        i32.const 1
        i32.add))
    "#;
    run(module_wat)
}
