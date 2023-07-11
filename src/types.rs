use anyhow::{anyhow, bail, Result};
use function_name::named;
use massa_proto_rs::massa::model::v1::{
    AddressCategory, ComparisonResult, NativeAmount, NativeTime, Slot,
};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    collections::{BTreeSet, HashMap},
    path::PathBuf,
};

use crate::execution::RuntimeModule;

/// That's what is returned when a module is executed correctly since the end
#[derive(Debug)]
pub struct Response {
    /// returned value from the module call
    pub ret: Vec<u8>,
    /// number of gas that remain after the execution (metering)
    pub remaining_gas: u64,
    /// number of gas required for the instance creation
    pub init_gas_cost: u64,
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

#[derive(Clone, Debug)]
pub struct GasCosts {
    pub(crate) operator_cost: u64,
    pub(crate) launch_cost: u64,
    pub(crate) abi_costs: HashMap<String, u64>,
    pub sp_compilation_cost: u64,
}

impl GasCosts {
    pub fn new(abi_cost_file: PathBuf, wasm_abi_file: PathBuf) -> Result<Self> {
        let abi_cost_file = std::fs::read_to_string(abi_cost_file)?;
        let mut abi_costs: HashMap<String, u64> =
            serde_json::from_str(&abi_cost_file)?;
        abi_costs.iter_mut().for_each(|(_, v)| {
            let unit_digit = *v % 10;
            if unit_digit > 5 {
                *v += 10 - unit_digit;
            } else {
                *v -= unit_digit;
            }
        });
        let wasm_abi_file = std::fs::read_to_string(wasm_abi_file)?;
        let wasm_costs: HashMap<String, u64> =
            serde_json::from_str(&wasm_abi_file)?;
        Ok(Self {
            operator_cost: wasm_costs.values().copied().sum::<u64>()
                / wasm_costs.len() as u64,
            launch_cost: *abi_costs.get("launch").ok_or_else(|| {
                anyhow!("launch cost not found in ABI gas cost file.")
            })?,
            sp_compilation_cost: *abi_costs
                .get("sp_compilation_cost")
                .ok_or_else(|| {
                    anyhow!(
                        "sp_compilation_cost not found in ABI gas cost file."
                    )
                })?,
            abi_costs,
        })
    }
}

#[cfg(any(test, feature = "gas_calibration", feature = "testing"))]
impl Default for GasCosts {
    fn default() -> Self {
        let mut abi_costs = HashMap::new();
        abi_costs.insert(
            String::from("assembly_script_address_from_public_key"),
            147,
        );
        abi_costs.insert(String::from("assembly_script_validate_address"), 4);
        abi_costs.insert(String::from("assembly_script_append_data"), 162);
        abi_costs.insert(String::from("assembly_script_append_data_for"), 200);
        abi_costs.insert(String::from("assembly_script_call"), 30466);
        abi_costs.insert(String::from("assembly_script_create_sc"), 160);
        abi_costs.insert(String::from("assembly_script_delete_data"), 78);
        abi_costs.insert(String::from("assembly_script_delete_data_for"), 120);
        abi_costs.insert(String::from("assembly_script_generate_event"), 36);
        abi_costs.insert(String::from("assembly_script_get_balance"), 4);
        abi_costs.insert(String::from("assembly_script_get_balance_for"), 41);
        abi_costs.insert(String::from("assembly_script_get_call_coins"), 9);
        abi_costs.insert(String::from("assembly_script_get_call_stack"), 56);
        abi_costs.insert(String::from("assembly_script_get_current_slot"), 9);
        abi_costs.insert(String::from("assembly_script_get_data"), 85);
        abi_costs.insert(String::from("assembly_script_get_data_for"), 139);
        abi_costs.insert(String::from("assembly_script_get_keys"), 26);
        abi_costs.insert(String::from("assembly_script_get_keys_for"), 48);
        abi_costs.insert(String::from("assembly_script_get_op_data"), 71);
        abi_costs.insert(String::from("assembly_script_get_op_keys"), 138);
        abi_costs
            .insert(String::from("assembly_script_get_owned_addresses"), 52);
        abi_costs.insert(String::from("assembly_script_get_remaining_gas"), 7);
        abi_costs.insert(String::from("assembly_script_get_time"), 4);
        abi_costs.insert(String::from("assembly_script_has_data"), 69);
        abi_costs.insert(String::from("assembly_script_has_data_for"), 115);
        abi_costs.insert(String::from("assembly_script_has_op_key"), 78);
        abi_costs.insert(String::from("assembly_script_hash"), 83);
        abi_costs.insert(String::from("assembly_script_hash_sha256"), 83);
        abi_costs.insert(String::from("assembly_script_keccak256_hash"), 83);
        abi_costs.insert(String::from("assembly_script_print"), 35);
        abi_costs.insert(String::from("assembly_script_send_message"), 316);
        abi_costs.insert(
            String::from("assembly_script_get_origin_operation_id"),
            200,
        );
        abi_costs.insert(String::from("assembly_script_set_bytecode"), 74);
        abi_costs.insert(String::from("assembly_script_set_bytecode_for"), 129);
        abi_costs.insert(String::from("assembly_script_set_data"), 158);
        abi_costs.insert(String::from("assembly_script_set_data_for"), 165);
        abi_costs.insert(String::from("assembly_script_signature_verify"), 98);
        abi_costs
            .insert(String::from("assembly_script_evm_signature_verify"), 264);
        abi_costs.insert(String::from("assembly_script_transfer_coins"), 62);
        abi_costs
            .insert(String::from("assembly_script_transfer_coins_for"), 102);
        abi_costs.insert(String::from("assembly_script_unsafe_random"), 11);
        abi_costs.insert(String::from("assembly_script_call"), 11);
        abi_costs.insert(String::from("assembly_script_local_call"), 11);
        abi_costs.insert(String::from("assembly_script_local_execution"), 11);
        abi_costs.insert(String::from("assembly_script_get_bytecode"), 11);
        abi_costs.insert(String::from("assembly_script_get_bytecode_for"), 11);
        abi_costs.insert(
            String::from("assembly_script_caller_has_write_access"),
            11,
        );
        abi_costs.insert(String::from("assembly_script_function_exists"), 11);
        abi_costs.insert(String::from("assembly_script_seed"), 11);
        abi_costs.insert(String::from("assembly_script_abort"), 11);
        abi_costs.insert(String::from("assembly_script_date_now"), 11);
        abi_costs.insert(String::from("assembly_script_console_log"), 36); // same cost as for generate_event
        abi_costs.insert(String::from("assembly_script_console_info"), 36);
        abi_costs.insert(String::from("assembly_script_console_debug"), 36);
        abi_costs.insert(String::from("assembly_script_console_warn"), 36);
        abi_costs.insert(String::from("assembly_script_console_error"), 36);
        abi_costs.insert(String::from("assembly_script_trace"), 36);
        Self {
            operator_cost: 1,
            launch_cost: 10_000,
            abi_costs,
            sp_compilation_cost: 10_000,
        }
    }
}

#[allow(unused_variables)]
pub trait Interface: Send + Sync + InterfaceClone {
    /// Prepare the execution of a module at the given address and transfer a
    /// given amount of coins
    fn init_call(&self, address: &str, raw_coins: u64) -> Result<Vec<u8>> {
        unimplemented!("init_call")
    }

    /// Prepare the execution of a module at the given address and transfer a
    /// given amount of coins
    fn init_call_wasmv1(
        &self,
        address: &str,
        raw_coins: NativeAmount,
    ) -> Result<Vec<u8>> {
        unimplemented!("init_call_wasmv1")
    }

    /// Finish a call
    fn finish_call(&self) -> Result<()> {
        unimplemented!("finish_call")
    }

    /// Get the SCE ledger balance for the current address.
    /// Defaults to zero if the address is not found.
    fn get_balance(&self) -> Result<u64> {
        unimplemented!("get_balance")
    }

    /// Get the SCE ledger balance for an address.
    /// Defaults to zero if the address is not found.
    fn get_balance_for(&self, address: &str) -> Result<u64> {
        unimplemented!("get_balance_for")
    }

    fn get_balance_wasmv1(
        &self,
        address: Option<String>,
    ) -> Result<NativeAmount> {
        unimplemented!("get_balance_wasmv1")
    }

    /// Transfer an amount from the address on the current call stack to a
    /// target address.
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

    fn transfer_coins_wasmv1(
        &self,
        to_address: String,
        raw_amount: NativeAmount,
        from_address: Option<String>,
    ) -> Result<()> {
        unimplemented!("transfer_coins_wasmv1")
    }

    /// Get the amount of coins that have been made available for use by the
    /// caller of the currently executing code.
    fn get_call_coins(&self) -> Result<u64> {
        bail!("unimplemented function get_call_coins_for in interface")
    }

    /// Get the native amount of coins that have been made available for use by
    /// the caller of the currently executing code.
    fn get_call_coins_wasmv1(&self) -> Result<NativeAmount>;

    /// Sets the executable bytecode at a current address.
    fn raw_set_bytecode(&self, bytecode: &[u8]) -> Result<()> {
        unimplemented!("raw_set_bytecode")
    }

    /// Sets the executable bytecode at a target address.
    /// The target address must exist and the current context must have access
    /// rights.
    fn raw_set_bytecode_for(
        &self,
        address: &str,
        bytecode: &[u8],
    ) -> Result<()> {
        unimplemented!("raw_set_bytecode_for")
    }

    #[named]
    fn set_bytecode_wasmv1(
        &self,
        bytecode: &[u8],
        address: Option<String>,
    ) -> Result<()> {
        unimplemented!(function_name!())
    }

    /// Requires a new address that contains the sent &[u8]
    fn create_module(&self, module: &[u8]) -> Result<String> {
        unimplemented!("create_module")
    }

    /// Print function for examples
    fn print(&self, message: &str) -> Result<()> {
        unimplemented!("print")
    }

    /// Return datastore keys
    /// Will only return keys with a given prefix if provided in args
    fn get_keys(&self, prefix: Option<&[u8]>) -> Result<BTreeSet<Vec<u8>>> {
        unimplemented!("get_op_keys")
    }

    /// Return datastore keys
    /// Will only return keys with a given prefix if provided in args
    fn get_keys_for(
        &self,
        address: &str,
        prefix: Option<&[u8]>,
    ) -> Result<BTreeSet<Vec<u8>>> {
        unimplemented!("get_op_keys_for")
    }

    #[named]
    fn get_ds_keys_wasmv1(
        &self,
        prefix: &[u8],
        address: Option<String>,
    ) -> Result<BTreeSet<Vec<u8>>> {
        unimplemented!(function_name!())
    }

    /// Return the datastore value of the corresponding key
    fn raw_get_data(&self, key: &[u8]) -> Result<Vec<u8>> {
        unimplemented!("raw_get_data")
    }

    /// Requires the data at the address
    fn raw_get_data_for(&self, address: &str, key: &[u8]) -> Result<Vec<u8>> {
        unimplemented!("raw_get_data_for")
    }

    #[named]
    fn get_ds_value_wasmv1(
        &self,
        key: &[u8],
        address: Option<String>,
    ) -> Result<Vec<u8>> {
        unimplemented!(function_name!())
    }

    /// Set the datastore value for the corresponding key
    fn raw_set_data(&self, key: &[u8], value: &[u8]) -> Result<()> {
        unimplemented!("raw_set_data")
    }

    /// Set the datastore value for the corresponding key of the given address
    fn raw_set_data_for(
        &self,
        address: &str,
        key: &[u8],
        value: &[u8],
    ) -> Result<()> {
        unimplemented!("raw_set_data_for")
    }

    #[named]
    fn set_ds_value_wasmv1(
        &self,
        key: &[u8],
        value: &[u8],
        address: Option<String>,
    ) -> Result<()> {
        unimplemented!(function_name!())
    }

    /// Append a value to the current datastore value for the corresponding key
    fn raw_append_data(&self, key: &[u8], value: &[u8]) -> Result<()> {
        unimplemented!("raw_append_data")
    }

    /// Append a value to the current datastore value for the corresponding key
    /// and the given address
    fn raw_append_data_for(
        &self,
        address: &str,
        key: &[u8],
        value: &[u8],
    ) -> Result<()> {
        unimplemented!("raw_append_data_for")
    }

    #[named]
    fn append_ds_value_wasmv1(
        &self,
        key: &[u8],
        value: &[u8],
        address: Option<String>,
    ) -> Result<()> {
        unimplemented!(function_name!())
    }

    /// Delete a datastore entry
    fn raw_delete_data(&self, key: &[u8]) -> Result<()> {
        unimplemented!("raw_delete_data")
    }

    /// Delete a datastore entry at of the given address
    fn raw_delete_data_for(&self, address: &str, key: &[u8]) -> Result<()> {
        unimplemented!("raw_delete_data_for")
    }

    #[named]
    fn delete_ds_entry_wasmv1(
        &self,
        key: &[u8],
        address: Option<String>,
    ) -> Result<()> {
        unimplemented!(function_name!())
    }

    /// Requires to replace the data in the current address
    ///
    /// Note:
    /// The execution lib will always use the current context address for the
    /// update
    fn has_data(&self, key: &[u8]) -> Result<bool> {
        unimplemented!("has_data")
    }

    /// Check if a datastore entry exists
    fn has_data_for(&self, address: &str, key: &[u8]) -> Result<bool> {
        unimplemented!("has_data_for")
    }

    #[named]
    fn ds_entry_exists_wasmv1(
        &self,
        key: &[u8],
        address: Option<String>,
    ) -> Result<bool> {
        unimplemented!(function_name!())
    }

    /// Returns bytecode of the current address
    fn raw_get_bytecode(&self) -> Result<Vec<u8>> {
        unimplemented!("raw_get_bytecode")
    }

    /// Returns bytecode of the target address
    fn raw_get_bytecode_for(&self, address: &str) -> Result<Vec<u8>> {
        unimplemented!("raw_get_bytecode_for")
    }

    #[named]
    fn get_bytecode_wasmv1(&self, address: Option<String>) -> Result<Vec<u8>> {
        unimplemented!(function_name!())
    }

    /// Return operation datastore keys
    fn get_op_keys(&self) -> Result<Vec<Vec<u8>>> {
        unimplemented!("get_op_keys")
    }

    fn get_op_keys_wasmv1(&self, prefix: &[u8]) -> Result<Vec<Vec<u8>>> {
        unimplemented!("get_op_keys_wasmv1")
    }

    /// Check if key is in operation datastore
    fn has_op_key(&self, key: &[u8]) -> Result<bool> {
        unimplemented!("has_op_data")
    }

    /// Return operation datastore data for a given key
    fn get_op_data(&self, key: &[u8]) -> Result<Vec<u8>> {
        unimplemented!("get_op_data")
    }

    /// Check whether or not the caller has write access in the current context
    fn caller_has_write_access(&self) -> Result<bool> {
        unimplemented!("caller_has_write_access")
    }

    // Hash data
    fn hash(&self, data: &[u8]) -> Result<[u8; 32]> {
        unimplemented!("hash")
    }

    /// Returns the blake3 hash of the given bytes
    #[named]
    fn hash_blake3(&self, bytes: &[u8]) -> Result<[u8; 32]> {
        unimplemented!(function_name!())
    }

    // Verify signature
    fn signature_verify(
        &self,
        data: &[u8],
        signature: &str,
        public_key: &str,
    ) -> Result<bool> {
        unimplemented!("signature_verify")
    }

    // Verify EVM signature
    fn verify_evm_signature(
        &self,
        message: &[u8],
        signature: &[u8],
        public_key: &[u8],
    ) -> Result<bool> {
        unimplemented!("verify_evm_signature")
    }

    // Convert a public key to an address
    fn address_from_public_key(&self, public_key: &str) -> Result<String> {
        unimplemented!("address_from_public_key")
    }

    // Validate an address
    fn validate_address(&self, address: &str) -> Result<bool> {
        unimplemented!("validate_address")
    }

    /// Returns the current time (millisecond unix timestamp)
    fn get_time(&self) -> Result<u64> {
        unimplemented!("get_time")
    }

    /// Returns a random number (unsafe: can be predicted and manipulated)
    fn unsafe_random(&self) -> Result<i64> {
        unimplemented!("unsafe_random")
    }

    /// Returns a random number (unsafe: can be predicted and manipulated)
    fn unsafe_random_f64(&self) -> Result<f64> {
        unimplemented!("unsafe_random_f64")
    }

    /// Returns a random number (unsafe: can be predicted and manipulated)
    fn unsafe_random_wasmv1(&self, num_bytes: u64) -> Result<Vec<u8>> {
        unimplemented!("unsafe_random_wasmv1")
    }

    /// Returns the period of the current execution slot
    fn get_current_period(&self) -> Result<u64> {
        unimplemented!("get_current_period")
    }

    /// Returns the thread of the current execution slot
    fn get_current_thread(&self) -> Result<u8> {
        unimplemented!("get_current_thread")
    }

    /// Returns the current execution slot
    fn get_current_slot(&self) -> Result<Slot> {
        unimplemented!("get_current_slot")
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

    /// Generate a smart contract event
    fn generate_event(&self, _event: String) -> Result<()> {
        unimplemented!("generate_event")
    }

    /// Generate a smart contract event
    fn generate_event_wasmv1(&self, _event: Vec<u8>) -> Result<()> {
        unimplemented!("generate_event_wasmv1")
    }

    /// For the given bytecode:
    ///
    /// * Get the corresponding runtime module if it already exists
    /// * Compile it if not
    fn get_module(&self, bytecode: &[u8], limit: u64) -> Result<RuntimeModule> {
        unimplemented!("get_module")
    }

    /// Sends an async message
    ///
    /// # Arguments
    ///
    /// * `target_address` - Destination address hash in format string
    /// * `target_handler` - Name of the message handling function
    /// * `validity_start` - Tuple containing the period and thread of the
    ///   validity start slot
    /// * `validity_end` - Tuple containing the period and thread of the
    ///   validity end slot
    /// * `max_gas` - Maximum gas for the message execution
    /// * `raw_fee` - Fee to be paid for message execution
    /// * `coins` - Coins of the sender
    /// * `data` - Message data
    #[allow(clippy::too_many_arguments)]
    fn send_message(
        &self,
        target_address: &str,
        target_handler: &str,
        validity_start: (u64, u8),
        validity_end: (u64, u8),
        max_gas: u64,
        raw_fee: u64,
        raw_coins: u64,
        data: &[u8],
        filter: Option<(&str, Option<&[u8]>)>,
    ) -> Result<()> {
        unimplemented!("send_message")
    }

    // Returns the operation id that originated the current execution if there
    // is one
    fn get_origin_operation_id(&self) -> Result<Option<String>> {
        unimplemented!("get_origin_operation_id")
    }

    // Sha256 hash bytes
    fn hash_sha256(&self, bytes: &[u8]) -> Result<[u8; 32]> {
        unimplemented!("hash_sha256")
    }

    // Keccak256 hash bytes
    fn hash_keccak256(&self, bytes: &[u8]) -> Result<[u8; 32]> {
        unimplemented!("hash_keccak256")
    }

    fn native_amount_from_str_wasmv1(
        &self,
        amount: &str,
    ) -> Result<NativeAmount> {
        unimplemented!("native_amount_from_str_wasmv1");
    }

    fn native_amount_to_string_wasmv1(
        &self,
        amount: &NativeAmount,
    ) -> Result<String> {
        unimplemented!("native_amount_to_string_wasmv1");
    }

    fn check_native_amount_wasmv1(
        &self,
        amount: &NativeAmount,
    ) -> Result<bool> {
        unimplemented!("check_native_amount_wasmv1");
    }

    fn add_native_amounts_wasmv1(
        &self,
        amount1: &NativeAmount,
        amount2: &NativeAmount,
    ) -> Result<NativeAmount> {
        unimplemented!("add_native_amounts_wasmv1");
    }

    fn sub_native_amounts_wasmv1(
        &self,
        amount1: &NativeAmount,
        amount2: &NativeAmount,
    ) -> Result<NativeAmount> {
        unimplemented!("sub_native_amounts_wasmv1");
    }

    fn mul_native_amount_wasmv1(
        &self,
        amount: &NativeAmount,
        factor: u64,
    ) -> Result<NativeAmount> {
        unimplemented!("mul_native_amount_wasmv1");
    }

    fn div_rem_native_amount_wasmv1(
        &self,
        dividend: &NativeAmount,
        divisor: u64,
    ) -> Result<(NativeAmount, NativeAmount)> {
        unimplemented!("div_rem_native_amount_wasmv1");
    }

    fn div_rem_native_amounts_wasmv1(
        &self,
        dividend: &NativeAmount,
        divisor: &NativeAmount,
    ) -> Result<(u64, NativeAmount)> {
        unimplemented!("div_rem_native_amounts_wasmv1");
    }
    fn check_address_wasmv1(&self, to_check: &String) -> Result<bool> {
        unimplemented!("check_address_wasmv1");
    }

    fn check_pubkey_wasmv1(&self, to_check: &String) -> Result<bool> {
        unimplemented!("check_pubkey_wasmv1");
    }

    fn check_signature_wasmv1(&self, to_check: &String) -> Result<bool> {
        unimplemented!("check_signature_wasmv1");
    }

    fn get_address_category_wasmv1(
        &self,
        to_check: &String,
    ) -> Result<AddressCategory> {
        unimplemented!("get_address_category_wasmv1");
    }

    fn get_address_version_wasmv1(&self, address: &String) -> Result<u64> {
        unimplemented!("get_address_version_wasmv1");
    }

    fn get_pubkey_version_wasmv1(&self, pubkey: &String) -> Result<u64> {
        unimplemented!("get_pubkey_version_wasmv1");
    }

    fn get_signature_version_wasmv1(&self, signature: &String) -> Result<u64> {
        unimplemented!("get_signature_version_wasmv1");
    }

    fn checked_add_native_time_wasmv1(
        &self,
        time1: &NativeTime,
        time2: &NativeTime,
    ) -> Result<NativeTime> {
        unimplemented!("checked_add_native_time_wasmv1");
    }

    fn checked_sub_native_time_wasmv1(
        &self,
        time1: &NativeTime,
        time2: &NativeTime,
    ) -> Result<NativeTime> {
        unimplemented!("checked_sub_native_time_wasmv1");
    }

    fn checked_mul_native_time_wasmv1(
        &self,
        time: &NativeTime,
        factor: u64,
    ) -> Result<NativeTime> {
        unimplemented!("checked_mul_native_time_wasmv1");
    }

    fn checked_scalar_div_native_time_wasmv1(
        &self,
        dividend: &NativeTime,
        divisor: u64,
    ) -> Result<(NativeTime, NativeTime)> {
        unimplemented!("checked_scalar_div_native_time_wasmv1");
    }

    fn checked_div_native_time_wasmv1(
        &self,
        dividend: &NativeTime,
        divisor: &NativeTime,
    ) -> Result<(u64, NativeTime)> {
        unimplemented!("checked_div_native_time_wasmv1");
    }

    fn base58_check_to_bytes_wasmv1(&self, s: &str) -> Result<Vec<u8>>;

    fn bytes_to_base58_check_wasmv1(&self, bytes: &[u8]) -> String;

    fn compare_address_wasmv1(
        &self,
        left: &str,
        right: &str,
    ) -> Result<ComparisonResult>;

    fn compare_native_amount_wasmv1(
        &self,
        left: &NativeAmount,
        right: &NativeAmount,
    ) -> Result<ComparisonResult>;

    fn compare_native_time_wasmv1(
        &self,
        left: &NativeTime,
        right: &NativeTime,
    ) -> Result<ComparisonResult>;

    fn compare_pub_key_wasmv1(
        &self,
        left: &str,
        right: &str,
    ) -> Result<ComparisonResult>;
}

impl dyn Interface {
    pub fn get_data<T: DeserializeOwned>(&self, key: &[u8]) -> Result<T> {
        Ok(serde_json::from_str::<T>(std::str::from_utf8(
            &self.raw_get_data(key)?,
        )?)?)
    }

    pub fn get_data_for<T: DeserializeOwned>(
        &self,
        address: &str,
        key: &[u8],
    ) -> Result<T> {
        Ok(serde_json::from_str::<T>(std::str::from_utf8(
            &self.raw_get_data_for(address, key)?,
        )?)?)
    }

    pub fn set_data<T: Serialize>(&self, key: &[u8], value: &T) -> Result<()> {
        // TODO: Avoid using this many conversions, protobuf serialization
        // should be enough
        self.raw_set_data(key, serde_json::to_string::<T>(value)?.as_bytes())
    }

    pub fn set_data_for<T: Serialize>(
        &self,
        address: &str,
        key: &[u8],
        value: &T,
    ) -> Result<()> {
        self.raw_set_data_for(
            address,
            key,
            serde_json::to_string::<T>(value)?.as_bytes(),
        )
    }
}
