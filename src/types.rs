use anyhow::{bail, Result};
use serde::{de::DeserializeOwned, Serialize};

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

macro_rules! unimplemented {
    ($fn: expr) => {
        bail!(format!("unimplemented function {} in interface", $fn))
    };
}

#[allow(unused_variables)]
pub trait Interface: Send + Sync + InterfaceClone {
    /// Prepare the execution of a module at the given address and transfer a given amount of coins
    fn init_call(&self, address: &str, raw_coins: u64) -> Result<Vec<u8>> {
        unimplemented!("init_call")
    }

    /// Finish a call
    fn finish_call(&self) -> Result<()> {
        unimplemented!("finish_call")
    }

    /// Requires the module in the given address
    fn get_module(&self, address: &str) -> Result<Vec<u8>> {
        unimplemented!("get_module")
    }

    /// Get the SCE ledger balance for the crrent address.
    /// Defaults to zero if the address is not found.
    fn get_balance(&self) -> Result<u64> {
        unimplemented!("get_balance")
    }

    /// Get the SCE ledger balance for an address.
    /// Defaults to zero if the address is not found.
    fn get_balance_for(&self, address: &str) -> Result<u64> {
        unimplemented!("get_balance_for")
    }

    /// Transfer an amount from the address on the current call stack to a target address.
    fn transfer_coins(&self, to_address: &str, raw_amount: u64) -> Result<()> {
        unimplemented!("transfer_coins")
    }

    /// Transfer an amount from the specified address to a target address.
    fn transfer_coins_for(
        &self,
        from_address: &str,
        to_address: &str,
        raw_amount: u64,
    ) -> Result<()> {
        unimplemented!("transfer_coins_for")
    }

    /// Get the amount of coins that have been made available for use by the caller of the currently executing code.
    fn get_call_coins(&self) -> Result<u64> {
        bail!("unimplemented function get_call_coins_for in interface")
    }

    /// Requires to replace the module at the current address
    ///
    /// Note:
    /// The execution lib will use the current context address for the update
    /// module and the new &[u8]
    fn update_module(&self, address: &str, module: &[u8]) -> Result<()> {
        unimplemented!("update_module")
    }

    /// Requires a new address that contains the &[u8] sended
    fn create_module(&self, module: &[u8]) -> Result<String> {
        unimplemented!("create_module")
    }

    /// Print function for examples
    fn print(&self, message: &str) -> Result<()> {
        unimplemented!("print")
    }

    fn raw_get_data(&self, key: &str) -> Result<Vec<u8>> {
        unimplemented!("raw_get_data")
    }

    fn raw_set_data(&self, key: &str, value: &[u8]) -> Result<()> {
        unimplemented!("raw_set_data")
    }

    /// Requires the data at the address
    fn raw_get_data_for(&self, address: &str, key: &str) -> Result<Vec<u8>> {
        unimplemented!("get_data_for")
    }

    fn raw_set_data_for(&self, address: &str, key: &str, value: &[u8]) -> Result<()> {
        unimplemented!("set_data_for")
    }

    /// Requires to replace the data in the current address
    ///
    /// Note:
    /// The execution lib will allways use the current context address for the update
    fn has_data(&self, key: &str) -> Result<bool> {
        unimplemented!("has_data")
    }

    fn has_data_for(&self, address: &str, _key: &str) -> Result<bool> {
        unimplemented!("has_data_for")
    }

    // Hash data
    fn hash(&self, data: &[u8]) -> Result<String> {
        unimplemented!("hash")
    }

    // Verify signature
    fn signature_verify(&self, data: &[u8], signature: &str, public_key: &str) -> Result<bool> {
        unimplemented!("signature_verify")
    }

    // Convert a public key to an address
    fn address_from_public_key(&self, public_key: &str) -> Result<String> {
        unimplemented!("address_from_public_key")
    }

    /// Returns the current time (millisecond unix timestamp)
    fn get_time(&self) -> Result<u64> {
        unimplemented!("get_time")
    }

    /// Returns a random number (unsafe: can be predicted and manipulated)
    fn unsafe_random(&self) -> Result<i64> {
        unimplemented!("unsafe_random")
    }

    fn module_called(&self) -> Result<()> {
        unimplemented!("module_called")
    }

    fn exit_success(&self) -> Result<()> {
        unimplemented!("exit_success")
    }

    /// Expect to return a list of owned addresses
    ///
    /// Required on smart-contract execute the imported function
    /// `assembly_script_get_owned_addresses`
    fn get_owned_addresses(&self) -> Result<Vec<String>> {
        unimplemented!("get_owned_addresses")
    }

    /// Expect to return a list of addresses in the call stack
    ///
    /// Required on smart-contract execute the imported function
    /// `assembly_script_get_call_stack`
    fn get_call_stack(&self) -> Result<Vec<String>> {
        unimplemented!("get_call_stack")
    }

    // TODO should be a SCEvent
    fn generate_event(&self, _event: String) -> Result<()> {
        unimplemented!("generate_event")
    }

    /// Sends an async message
    ///
    /// # Arguments
    ///
    /// * `target_address` - Destination address hash in format string
    /// * `target_handler` - Name of the message handling function
    /// * `validity_start` - Tuple containing the period and thread of the validity start slot
    /// * `validity_end` - Tuple containing the period and thread of the validity end slot
    /// * `max_gas` - Maximum gas for the message execution
    /// * `gas_price` - Price of one gas unit
    /// * `coins` - Coins of the sender
    /// * `payload` - Message payload
    /// 
    fn send_message(
        &self,
        target_address: &str,
        target_handler: &str,
        validity_start: (u64, u64),
        validity_end: (u64, u64),
        max_gas: u64,
        gas_price: &str,
        coins: &str,
        payload: &str,
    ) -> Result<()> {
        unimplemented!("send_message")
    }
}

impl dyn Interface {
    pub fn get_data<T: DeserializeOwned>(&self, key: &str) -> Result<T> {
        Ok(serde_json::from_str::<T>(std::str::from_utf8(
            &self.raw_get_data(key)?,
        )?)?)
    }

    pub fn set_data<T: Serialize>(&self, key: &str, value: &T) -> Result<()> {
        self.raw_set_data(key, serde_json::to_string::<T>(value)?.as_bytes())
    }

    pub fn get_data_for<T: DeserializeOwned>(&self, address: &str, key: &str) -> Result<T> {
        Ok(serde_json::from_str::<T>(std::str::from_utf8(
            &self.raw_get_data_for(address, key)?,
        )?)?)
    }

    pub fn set_data_for<T: Serialize>(&self, address: &str, key: &str, value: &T) -> Result<()> {
        self.raw_set_data_for(address, key, serde_json::to_string::<T>(value)?.as_bytes())
    }
}
