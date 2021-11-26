use std::fmt;
use wasmer::{Array, Memory, WasmPtr, WasmerEnv, Instance, HostEnvInitError, LazyInit};

pub type AsmScriptStringPtr = WasmPtr<u8, Array>;

#[derive(Clone)]
pub struct Env {
    pub memory: LazyInit<Memory>,
}

impl Default for Env {
    fn default() -> Self {
        Self { memory: LazyInit::default() }
    }
}

impl WasmerEnv for Env {
    fn init_with_instance(&mut self, instance: &Instance) -> Result<(), HostEnvInitError> {
        self.memory.initialize(
            instance
                .exports
                .get_memory("memory")
                .map_err(HostEnvInitError::from)?
                .clone(),
        );
        Ok(())
    }
}

// This is the main
pub fn abort(
    env: &Env,
    message: AsmScriptStringPtr,
    filename: AsmScriptStringPtr,
    line: i32,
    col: i32
) {
    let memory = env.memory.get_ref().expect("initialized memory");
    let message = message.read(memory).unwrap();
    let filename = filename.read(memory).unwrap();
    eprintln!("Error: {} at {}:{} col: {}", message, filename, line, col);
}

pub trait AsmScriptRead<T> {
    fn read(&self, memory: &Memory) -> Result<T, Error>;
    fn size(&self, memory: &Memory) -> Result<u32, Error>;
}

impl AsmScriptRead<String> for AsmScriptStringPtr {
    fn read(&self, memory: &Memory) -> Result<String, Error> {
        let size = self.size(memory)?;
        println!("offset: {}, size: {}", self.offset(), size);
        if let Some(buf) = self.deref(memory, 0, size) {
            let input: Vec<u8> = buf.iter().map(|b| b.get()).collect();
            println!("{:?}", input);
            Ok(String::from_utf8_lossy(&input).to_string())
        } else {
            Err(Error::Mem("Wrong offset: can't read buf"))
        }
    }

    fn size(&self, memory: &Memory) -> Result<u32, Error> {
        let offset = self.offset();
        if offset < 4 {
            return Err(Error::Mem("Wrong offset: less than 4"));
        }
        // read -4 offset
        // assemblyscript counts bytes
        // https://www.assemblyscript.org/memory.html#internals
        if let Some(cell) = memory.view::<u32>().get(offset as usize / (32 / 8) - 1) {
            Ok(cell.get())
        } else {
            Err(Error::Mem("Wrong offset: can't read size"))
        }
    }
}

//pub fn allocate_string(memory: &Memory, string: &str) -> AsmScriptStringPtr {
//    //let atomic_view = memory.view().atomically();
//    //for byte in atomic_view[0x1000 .. 0x1010].iter().map(|atom| atom.load(Ordering::SeqCst)) {
//    //    println!("byte: {}", byte);
//    //}
//    todo!()
//}

#[derive(Debug)]
pub enum Error {
    Mem(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Mem(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for Error {}
