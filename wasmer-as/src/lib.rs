use std::fmt;
use wasmer::{Array, Memory, WasmPtr, WasmerEnv, Instance, HostEnvInitError, LazyInit};

pub type AsmScriptStringPtr = WasmPtr<u16, Array>;

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
    fn read(self, memory: &Memory) -> Result<T, Error>;

    fn size(offset: u32, memory: &Memory) -> Result<u32, Error>;
}

impl AsmScriptRead<String> for AsmScriptStringPtr {
    fn read(self, memory: &Memory) -> Result<String, Error> {
        let offset = self.offset();
        let size = Self::size(offset, memory)?;

        // we need size / 2 because assemblyscript counts bytes
        // while deref considers u16 elements
        if let Some(buf) = self.deref(memory, 0, size / 2) {
            let input: Vec<u16> = buf.iter().map(|b| b.get()).collect();
            Ok(String::from_utf16_lossy(&input))
        } else {
            Err(Error::Mem("Wrong offset: can't read buf"))
        }
    }

    fn size(offset: u32, memory: &Memory) -> Result<u32, Error> {
        if offset < 4 {
            return Err(Error::Mem("Wrong offset: less than 2"));
        }

        // read -4 offset
        // https://www.assemblyscript.org/memory.html#internals
        if let Some(cell) = memory.view::<u32>().get(offset as usize / (32 / 8) - 1) {
            Ok(cell.get())
        } else {
            Err(Error::Mem("Wrong offset: can't read size"))
        }
    }
}

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
