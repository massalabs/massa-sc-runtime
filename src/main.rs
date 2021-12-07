use anyhow::{bail, Result};
use execution::run;
use std::env;
use std::fs;
use std::path::Path;
mod interface;

fn read_files() -> Result<Vec<(String, Vec<u8>)>> {
    // TODO: should be or use a read_files(filename: Path) -> String
    let args: Vec<String> = env::args().collect();
    let mut ret = vec![];
    #[allow(clippy::needless_range_loop)]
    for i in 1..args.len() {
        let name = args[i].clone();
        let path = Path::new(&name);
        if !path.is_file() {
            bail!("{} isn't file", name)
        }
        // TODO: should also handle binary WASM file?!
        let extention = path.extension().unwrap_or_default();
        if extention != "wat" && extention != "wasm" {
            bail!("{} should be in webassembly", name)
        }
        ret.push((path.to_str().unwrap().to_string(), fs::read(path)?));
    }
    Ok(ret)
}

fn main() -> Result<()> {
    let modules = read_files()?;
    for (address, module) in modules.into_iter() {
        run(address, &module, 20000, &interface::new())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
