use anyhow::{bail, Result};
use std::time::Instant;

pub type Address = String;
pub type Bytecode = Vec<u8>;

/// That's what is returned when a module is executed correctly since the end
pub(crate) struct Response {
    /// returned value from the module call
    pub ret: String,
    /// number of points that remain after the execution (metering)
    pub remaining_points: u64,
}

#[derive(Clone)]
pub struct Interface {
    /// Requires the module in the given address
    pub get_module: fn(address: &Address) -> Result<Bytecode>,
    /// Requires to replace the module at the current address
    ///
    /// Note:
    /// The execution lib will use the current context address for the update
    /// module and the new bytecode
    pub update_module: fn(address: &Address, module: &Bytecode) -> Result<()>,
    /// Requires a new address that contains the bytecode sended
    pub create_module: fn(module: &Bytecode) -> Result<Address>,
    /// Requires the data at the address
    pub get_data: fn(address: &Address, key: &str) -> Result<Bytecode>,
    /// Requires to replace the data in the current address
    ///
    /// Note:
    /// The execution lib will allways use the current context address for the update
    pub set_data: fn(address: &Address, key: &str, value: &Bytecode) -> Result<()>,
    /// Requires a time
    pub get_time: fn() -> Result<Instant>,
    /// Requires a random number
    pub get_random: fn() -> Result<u64>,
    /// Print function
    pub print: fn(message: &str) -> Result<()>,
}

impl Default for Interface {
    fn default() -> Self {
        Self {
            get_module: |_| bail!("unimplemented function get_module in interface"),
            update_module: |_, _| bail!("unimplemented function update_module in interface"),
            create_module: |_| bail!("unimplemented function create_module in interface"),
            get_data: |_, _| bail!("unimplemented function get_data in interface"),
            set_data: |_, _, _| bail!("unimplemented function set_data in interface"),
            get_time: || bail!("unimplemented function get_time in interface"),
            get_random: || bail!("unimplemented function get_random in interface"),
            print: |_| bail!("unimplemented function print in interface"),
        }
    }
}
