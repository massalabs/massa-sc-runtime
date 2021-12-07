use anyhow::{bail, Result};

pub type Address = String;
pub type Bytecode = Vec<u8>;
pub type Ledger = std::collections::BTreeMap<Address, Bytecode>; // Byttecode instead of String

/// That's what is returned when a module is executed correctly since the end
pub struct Response {
    /// returned value from the module call
    pub ret: String,
    /// number of points that remain after the execution (metering)
    pub remaining_points: u64,
}

#[derive(Clone)]
pub struct Interface {
    pub get_module: fn(address: &Address) -> Result<Bytecode>,
    pub update_module: fn(address: &Address, module: &Bytecode) -> Result<()>,
    pub create_module: fn(address: &Address) -> Result<Bytecode>,
}

impl Default for Interface {
    fn default() -> Self {
        Self {
            get_module: |_| bail!("unimplemented function"),
            update_module: |_, _| bail!("unimplemented function"),
            create_module: |_| bail!("unimplemented function"),
        }
    }
}
