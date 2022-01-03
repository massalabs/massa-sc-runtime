use anyhow::{bail, Result};
use std::time::Instant;

pub type Address = String;
pub type Bytecode = Vec<u8>;

/// That's what is returned when a module is executed correctly since the end
pub(crate) struct Response {
    /// returned value from the module call
    pub ret: String,
    /// number of gas that remain after the execution (metering)
    pub remaining_gas: u64,
}

pub trait InterfaceClone {
    fn clone_box(&self) -> Box<dyn Interface>;
}

impl Clone for Box<dyn Interface> {
    fn clone(&self) -> Box<dyn Interface> {
        self.clone_box()
    }
}

pub trait Interface: Send + Sync + InterfaceClone {
    /// Requires the module in the given address
    fn get_module(&self, _address: &Address) -> Result<Bytecode> {
        bail!("unimplemented function get_module in interface")
    }

    /// Requires to replace the module at the current address
    ///
    /// Note:
    /// The execution lib will use the current context address for the update
    /// module and the new bytecode
    fn update_module(&self, _address: &Address, _module: &Bytecode) -> Result<()> {
        bail!("unimplemented function update_module in interface")
    }

    /// Requires a new address that contains the bytecode sended
    fn create_module(&self, _module: &Bytecode) -> Result<Address> {
        bail!("unimplemented function create_module in interface")
    }

    /// Requires the data at the address
    fn get_data_for(&self, _address: &Address, _key: &str) -> Result<Bytecode> {
        bail!("unimplemented function get_data_for in interface")
    }

    /// Print function for examples
    fn print(&self, _message: &str) -> Result<()> {
        bail!("unimplemented function print in interface")
    }

    /// Requires to replace the data in the current address
    ///
    /// Note:
    /// The execution lib will allways use the current context address for the update
    fn set_data_for(&self, _address: &Address, _key: &str, _value: &Bytecode) -> Result<()> {
        bail!("unimplemented function set_data_for in interface")
    }

    fn get_data(&self, _key: &str) -> Result<Bytecode> {
        bail!("unimplemented function get_data in interface")
    }

    fn set_data(&self, _key: &str, _value: &Bytecode) -> Result<()> {
        bail!("unimplemented function set_data in interface")
    }

    /// Returns the current time
    fn get_time(&self) -> Result<Instant> {
        bail!("unimplemented function get_time in interface")
    }

    /// Returns a random number
    fn get_random(&self) -> Result<u64> {
        bail!("unimplemented function get_random in interface")
    }

    fn module_called(&self) -> Result<()> {
        bail!("unimplemented function module_called in interface")
    }

    fn exit_success(&self) -> Result<()> {
        bail!("unimplemented function exit_success in interface")
    }

    fn get_owned_addresses(&self) -> Result<Vec<Address>> {
        bail!("unimplemented function get_owned_addresses in interface")
    }

    fn get_call_stack(&self) -> Result<Vec<Address>> {
        bail!("unimplemented function get_call_stack in interface")
    }
}
