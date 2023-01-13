use schnellru::{ByLength, LruMap};
use wasmer::{CompileError, Engine, Module};

/// Cache sent to `run_function` calls to avoid recompiling frequently used modules
pub struct ModuleCache {
    lru_cache: LruMap<Vec<u8>, Module>,
}

impl ModuleCache {
    /// Creates a new `ModuleCache` with the given length
    pub fn new(len: u32) -> Self {
        Self {
            lru_cache: LruMap::new(ByLength::new(len)),
        }
    }

    /// If the module is contained in the cache:
    /// * retrieve a copy of it
    /// * move it up in the LRU cache
    ///
    /// If the module is not contained in the cache:
    /// * create the module
    /// * save the module in the cache
    /// * retrieve a copy of it
    pub(crate) fn get_module(
        &mut self,
        engine: &Engine,
        bytecode: &[u8],
    ) -> Result<Module, CompileError> {
        let module = if let Some(cached_module) = self.lru_cache.get(bytecode) {
            cached_module.clone()
        } else {
            let new_module = Module::new(engine, bytecode)?;
            self.lru_cache.insert(bytecode.to_vec(), new_module.clone());
            new_module
        };
        Ok(module)
    }
}
