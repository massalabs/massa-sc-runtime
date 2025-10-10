use massa_proto_rs::massa::model::v1::{
    AddressCategory, ComparisonResult, NativeAmount, NativeTime, Slot,
};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    collections::{BTreeSet, HashMap},
    path::PathBuf,
};

use crate::execution::RuntimeModule;

use displaydoc::Display;
use thiserror::Error;

#[derive(Error, Display, Debug)]
pub enum InterfaceError {
    /// IO error: {0}
    IoError(#[from] std::io::Error),
    /// Utf8 error: {0}
    Utf8Error(#[from] std::str::Utf8Error),
    /// Serde error: {0}
    SerdeError(#[from] serde_json::Error),
    /// Gas calibration error {0}
    GasCalibrationError(String),
    /// Interface generic error {0}
    GenericError(String),
    /// Interface depth error {0}
    DepthError(String),
}
pub type Result<T, E = InterfaceError> = core::result::Result<T, E>;

impl From<&str> for InterfaceError {
    fn from(msg: &str) -> Self {
        Self::GenericError(msg.to_owned())
    }
}

impl From<String> for InterfaceError {
    fn from(msg: String) -> Self {
        Self::GenericError(msg)
    }
}

#[macro_export]
macro_rules! bail {
    ($msg:literal $(,)?) => {
        return Err(InterfaceError::GenericError($msg.to_string()))
    };
    ($fmt:expr, $($arg:tt)*) => {
        return Err(InterfaceError::GenericError(format!($fmt, $($arg)*)))
    };
}

#[cfg(feature = "execution-trace")]
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", content = "value", rename_all = "camelCase")]
pub enum AbiTraceType {
    None,
    Bool(bool),
    U8(u8),
    I32(i32),
    U32(u32),
    I64(i64),
    U64(u64),
    F64(f64),
    ByteArray(Vec<u8>),
    ByteArrays(Vec<Vec<u8>>),
    String(String),
    Strings(Vec<String>),
    Slot((u64, u8)),
}

#[cfg(feature = "execution-trace")]
impl From<bool> for AbiTraceType {
    fn from(v: bool) -> Self {
        Self::Bool(v)
    }
}
#[cfg(feature = "execution-trace")]
impl From<u8> for AbiTraceType {
    fn from(v: u8) -> Self {
        Self::U8(v)
    }
}
#[cfg(feature = "execution-trace")]
impl From<i32> for AbiTraceType {
    fn from(v: i32) -> Self {
        Self::I32(v)
    }
}

#[cfg(feature = "execution-trace")]
impl From<u32> for AbiTraceType {
    fn from(v: u32) -> Self {
        Self::U32(v)
    }
}

#[cfg(feature = "execution-trace")]
impl From<i64> for AbiTraceType {
    fn from(v: i64) -> Self {
        Self::I64(v)
    }
}
#[cfg(feature = "execution-trace")]
impl From<u64> for AbiTraceType {
    fn from(v: u64) -> Self {
        Self::U64(v)
    }
}
#[cfg(feature = "execution-trace")]
impl From<f64> for AbiTraceType {
    fn from(v: f64) -> Self {
        Self::F64(v)
    }
}
#[cfg(feature = "execution-trace")]
impl From<Vec<u8>> for AbiTraceType {
    fn from(v: Vec<u8>) -> Self {
        Self::ByteArray(v)
    }
}
#[cfg(feature = "execution-trace")]
impl From<Vec<Vec<u8>>> for AbiTraceType {
    fn from(v: Vec<Vec<u8>>) -> Self {
        Self::ByteArrays(v)
    }
}
#[cfg(feature = "execution-trace")]
impl From<String> for AbiTraceType {
    fn from(v: String) -> Self {
        Self::String(v)
    }
}
#[cfg(feature = "execution-trace")]
impl From<Vec<String>> for AbiTraceType {
    fn from(v: Vec<String>) -> Self {
        Self::Strings(v)
    }
}

#[cfg(feature = "execution-trace")]
impl From<(u64, u8)> for AbiTraceType {
    fn from(v: (u64, u8)) -> Self {
        Self::Slot(v)
    }
}

#[cfg(feature = "execution-trace")]
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AbiTraceValue {
    pub name: String,
    #[serde(flatten)]
    pub value: AbiTraceType,
}

#[cfg(feature = "execution-trace")]
impl<T> From<(&str, T)> for AbiTraceValue
where
    T: Into<AbiTraceType>,
{
    fn from((name, value): (&str, T)) -> Self {
        Self {
            name: name.to_string(),
            value: value.into(),
        }
    }
}

#[cfg(feature = "execution-trace")]
#[macro_export]
macro_rules! into_trace_value {
    ($a: expr) => {{
        (stringify!($a), $a).into()
    }};
}

#[cfg(feature = "execution-trace")]
#[derive(Debug, Clone, PartialEq)]
pub struct AbiTrace {
    pub name: String,
    pub params: Vec<AbiTraceValue>,
    pub return_value: AbiTraceType,
    pub sub_calls: Option<Vec<AbiTrace>>,
}

/// That's what is returned when a module is executed correctly since the end
#[derive(Debug)]
pub struct Response {
    /// returned value from the module call
    pub ret: Vec<u8>,
    /// number of gas that remain after the execution (metering)
    pub remaining_gas: u64,
    /// number of gas required for the instance creation
    pub init_gas_cost: u64,
    #[cfg(feature = "execution-trace")]
    pub trace: Vec<AbiTrace>,
}

pub trait InterfaceClone {
    fn clone_box(&self) -> Box<dyn Interface>;
}

impl Clone for Box<dyn Interface> {
    fn clone(&self) -> Box<dyn Interface> {
        self.clone_box()
    }
}

#[derive(Clone, Debug, Default)]
pub struct CondomLimits {
    pub max_exports: Option<usize>,
    pub max_functions: Option<usize>,
    pub max_signature_len: Option<usize>,
    pub max_name_len: Option<usize>,
    pub max_imports_len: Option<usize>,
    pub max_table_initializers_len: Option<usize>,
    pub max_passive_elements_len: Option<usize>,
    pub max_passive_data_len: Option<usize>,
    pub max_global_initializers_len: Option<usize>,
    pub max_function_names_len: Option<usize>,
    pub max_tables_count: Option<usize>,
    pub max_memories_len: Option<usize>,
    pub max_globals_len: Option<usize>,
    pub max_custom_sections_len: Option<usize>,
    pub max_custom_sections_data_len: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct GasCosts {
    // Core costs
    pub(crate) launch_cost: u64,
    pub(crate) operator_cost: u64,
    pub cl_compilation_cost: u64,
    pub sp_compilation_cost: u64,
    pub max_instance_cost: u64,

    // AssemblyScript ABI costs (alphabetically sorted)
    pub assembly_script_abort: u64,
    pub assembly_script_address_from_public_key: u64,
    pub assembly_script_append_data: u64,
    pub assembly_script_append_data_for: u64,
    pub assembly_script_call: u64,
    pub assembly_script_caller_has_write_access: u64,
    pub assembly_script_chain_id: u64,
    pub assembly_script_console_debug: u64,
    pub assembly_script_console_error: u64,
    pub assembly_script_console_info: u64,
    pub assembly_script_console_log: u64,
    pub assembly_script_console_warn: u64,
    pub assembly_script_create_sc: u64,
    pub assembly_script_date_now: u64,
    pub assembly_script_deferred_call_cancel: u64,
    pub assembly_script_deferred_call_exists: u64,
    pub assembly_script_deferred_call_register: u64,
    pub assembly_script_delete_data: u64,
    pub assembly_script_delete_data_for: u64,
    pub assembly_script_evm_get_address_from_pubkey: u64,
    pub assembly_script_evm_get_pubkey_from_signature: u64,
    pub assembly_script_evm_signature_verify: u64,
    pub assembly_script_function_exists: u64,
    pub assembly_script_generate_event: u64,
    pub assembly_script_get_balance: u64,
    pub assembly_script_get_balance_for: u64,
    pub assembly_script_get_bytecode: u64,
    pub assembly_script_get_bytecode_for: u64,
    pub assembly_script_get_call_coins: u64,
    pub assembly_script_get_call_stack: u64,
    pub assembly_script_get_current_period: u64,
    pub assembly_script_get_current_thread: u64,
    pub assembly_script_get_data: u64,
    pub assembly_script_get_data_for: u64,
    pub assembly_script_get_deferred_call_quote: u64,
    pub assembly_script_get_keys: u64,
    pub assembly_script_get_keys_for: u64,
    pub assembly_script_get_op_data: u64,
    pub assembly_script_get_op_keys: u64,
    pub assembly_script_get_op_keys_prefix: u64,
    pub assembly_script_get_origin_operation_id: u64,
    pub assembly_script_get_owned_addresses: u64,
    pub assembly_script_get_remaining_gas: u64,
    pub assembly_script_get_time: u64,
    pub assembly_script_has_data: u64,
    pub assembly_script_has_data_for: u64,
    pub assembly_script_has_op_key: u64,
    pub assembly_script_hash: u64,
    pub assembly_script_hash_sha256: u64,
    pub assembly_script_is_address_eoa: u64,
    pub assembly_script_keccak256_hash: u64,
    pub assembly_script_local_call: u64,
    pub assembly_script_local_execution: u64,
    pub assembly_script_print: u64,
    pub assembly_script_seed: u64,
    pub assembly_script_send_message: u64,
    pub assembly_script_set_bytecode: u64,
    pub assembly_script_set_bytecode_for: u64,
    pub assembly_script_set_data: u64,
    pub assembly_script_set_data_for: u64,
    pub assembly_script_signature_verify: u64,
    pub assembly_script_trace: u64,
    pub assembly_script_transfer_coins: u64,
    pub assembly_script_transfer_coins_for: u64,
    pub assembly_script_unsafe_random: u64,
    pub assembly_script_validate_address: u64,

    // WasmV1 ABI costs
    pub abi_abort: u64,
    pub abi_add_native_amount: u64,
    pub abi_address_from_public_key: u64,
    pub abi_append_ds_value: u64,
    pub abi_base58_check_to_bytes: u64,
    pub abi_bytes_to_base58_check: u64,
    pub abi_call: u64,
    pub abi_caller_has_write_access: u64,
    pub abi_chain_id: u64,
    pub abi_check_address: u64,
    pub abi_check_native_amount: u64,
    pub abi_check_pubkey: u64,
    pub abi_check_signature: u64,
    pub abi_checked_add_native_time: u64,
    pub abi_checked_div_native_time: u64,
    pub abi_checked_mul_native_time: u64,
    pub abi_checked_scalar_div_native_time: u64,
    pub abi_checked_sub_native_time: u64,
    pub abi_compare_address: u64,
    pub abi_compare_native_amount: u64,
    pub abi_compare_native_time: u64,
    pub abi_compare_pub_key: u64,
    pub abi_create_sc: u64,
    pub abi_deferred_call_cancel: u64,
    pub abi_deferred_call_exists: u64,
    pub abi_deferred_call_register: u64,
    pub abi_delete_ds_entry: u64,
    pub abi_div_rem_native_amount: u64,
    pub abi_ds_entry_exists: u64,
    pub abi_evm_get_address_from_pubkey: u64,
    pub abi_evm_get_pubkey_from_signature: u64,
    pub abi_evm_verify_signature: u64,
    pub abi_function_exists: u64,
    pub abi_generate_event: u64,
    pub abi_get_address_category: u64,
    pub abi_get_address_version: u64,
    pub abi_get_balance: u64,
    pub abi_get_bytecode: u64,
    pub abi_get_call_coins: u64,
    pub abi_get_call_stack: u64,
    pub abi_get_current_slot: u64,
    pub abi_get_deferred_call_quote: u64,
    pub abi_get_ds_keys: u64,
    pub abi_get_ds_value: u64,
    pub abi_get_native_time: u64,
    pub abi_get_op_data: u64,
    pub abi_get_op_keys: u64,
    pub abi_get_origin_operation_id: u64,
    pub abi_get_owned_addresses: u64,
    pub abi_get_pubkey_version: u64,
    pub abi_get_remaining_gas: u64,
    pub abi_get_signature_version: u64,
    pub abi_hash_blake3: u64,
    pub abi_hash_keccak256: u64,
    pub abi_hash_sha256: u64,
    pub abi_is_address_eoa: u64,
    pub abi_local_call: u64,
    pub abi_local_execution: u64,
    pub abi_native_amount_from_string: u64,
    pub abi_native_amount_to_string: u64,
    pub abi_op_entry_exists: u64,
    pub abi_scalar_div_rem_native_amount: u64,
    pub abi_scalar_mul_native_amount: u64,
    pub abi_send_async_message: u64,
    pub abi_set_bytecode: u64,
    pub abi_set_ds_value: u64,
    pub abi_sub_native_amount: u64,
    pub abi_transfer_coins: u64,
    pub abi_unsafe_random: u64,
    pub abi_verify_signature: u64,
}

impl GasCosts {
    /// Round gas cost to nearest 10
    fn round_gas(v: u64) -> u64 {
        let unit_digit = v % 10;
        if unit_digit > 5 {
            v + (10 - unit_digit)
        } else {
            v - unit_digit
        }
    }

    pub fn new(abi_cost_file: PathBuf) -> Result<Self> {
        let abi_cost_file = std::fs::read_to_string(abi_cost_file)?;
        let mut abi_costs: HashMap<String, u64> = serde_json::from_str(&abi_cost_file)?;
        abi_costs.iter_mut().for_each(|(_, v)| {
            *v = Self::round_gas(*v);
        });

        // Helper macro to get and unwrap cost from HashMap
        macro_rules! get_cost {
            ($name:expr) => {
                *abi_costs.get($name).ok_or_else(|| {
                    InterfaceError::GasCalibrationError(
                        format!("{} cost not found in ABI gas cost file", $name).into(),
                    )
                })?
            };
        }

        Ok(Self {
            // Note: Use a constant = 23 here in order to not break compatibility with previous Massa node version
            //       The gas calibration for wasm operators is very incomplete for now and should be reworked
            //       See: https://github.com/massalabs/gas-calibration/issues/9
            operator_cost: 23,
            launch_cost: get_cost!("launch"),
            cl_compilation_cost: get_cost!("cl_compilation"),
            sp_compilation_cost: get_cost!("sp_compilation"),
            max_instance_cost: get_cost!("max_instance"),

            // AssemblyScript ABI costs
            assembly_script_abort: get_cost!("assembly_script_abort"),
            assembly_script_address_from_public_key: get_cost!(
                "assembly_script_address_from_public_key"
            ),
            assembly_script_append_data: get_cost!("assembly_script_append_data"),
            assembly_script_append_data_for: get_cost!("assembly_script_append_data_for"),
            assembly_script_call: get_cost!("assembly_script_call"),
            assembly_script_caller_has_write_access: get_cost!(
                "assembly_script_caller_has_write_access"
            ),
            assembly_script_chain_id: get_cost!("assembly_script_chain_id"),
            assembly_script_console_debug: get_cost!("assembly_script_console_debug"),
            assembly_script_console_error: get_cost!("assembly_script_console_error"),
            assembly_script_console_info: get_cost!("assembly_script_console_info"),
            assembly_script_console_log: get_cost!("assembly_script_console_log"),
            assembly_script_console_warn: get_cost!("assembly_script_console_warn"),
            assembly_script_create_sc: get_cost!("assembly_script_create_sc"),
            assembly_script_date_now: get_cost!("assembly_script_date_now"),
            assembly_script_deferred_call_cancel: get_cost!("assembly_script_deferred_call_cancel"),
            assembly_script_deferred_call_exists: get_cost!("assembly_script_deferred_call_exists"),
            assembly_script_deferred_call_register: get_cost!(
                "assembly_script_deferred_call_register"
            ),
            assembly_script_delete_data: get_cost!("assembly_script_delete_data"),
            assembly_script_delete_data_for: get_cost!("assembly_script_delete_data_for"),
            assembly_script_evm_get_address_from_pubkey: get_cost!(
                "assembly_script_evm_get_address_from_pubkey"
            ),
            assembly_script_evm_get_pubkey_from_signature: get_cost!(
                "assembly_script_evm_get_pubkey_from_signature"
            ),
            assembly_script_evm_signature_verify: get_cost!("assembly_script_evm_signature_verify"),
            assembly_script_function_exists: get_cost!("assembly_script_function_exists"),
            assembly_script_generate_event: get_cost!("assembly_script_generate_event"),
            assembly_script_get_balance: get_cost!("assembly_script_get_balance"),
            assembly_script_get_balance_for: get_cost!("assembly_script_get_balance_for"),
            assembly_script_get_bytecode: get_cost!("assembly_script_get_bytecode"),
            assembly_script_get_bytecode_for: get_cost!("assembly_script_get_bytecode_for"),
            assembly_script_get_call_coins: get_cost!("assembly_script_get_call_coins"),
            assembly_script_get_call_stack: get_cost!("assembly_script_get_call_stack"),
            assembly_script_get_current_period: get_cost!("assembly_script_get_current_period"),
            assembly_script_get_current_thread: get_cost!("assembly_script_get_current_thread"),
            assembly_script_get_data: get_cost!("assembly_script_get_data"),
            assembly_script_get_data_for: get_cost!("assembly_script_get_data_for"),
            assembly_script_get_deferred_call_quote: get_cost!(
                "assembly_script_get_deferred_call_quote"
            ),
            assembly_script_get_keys: get_cost!("assembly_script_get_keys"),
            assembly_script_get_keys_for: get_cost!("assembly_script_get_keys_for"),
            assembly_script_get_op_data: get_cost!("assembly_script_get_op_data"),
            assembly_script_get_op_keys: get_cost!("assembly_script_get_op_keys"),
            assembly_script_get_op_keys_prefix: get_cost!("assembly_script_get_op_keys_prefix"),
            assembly_script_get_origin_operation_id: get_cost!(
                "assembly_script_get_origin_operation_id"
            ),
            assembly_script_get_owned_addresses: get_cost!("assembly_script_get_owned_addresses"),
            assembly_script_get_remaining_gas: get_cost!("assembly_script_get_remaining_gas"),
            assembly_script_get_time: get_cost!("assembly_script_get_time"),
            assembly_script_has_data: get_cost!("assembly_script_has_data"),
            assembly_script_has_data_for: get_cost!("assembly_script_has_data_for"),
            assembly_script_has_op_key: get_cost!("assembly_script_has_op_key"),
            assembly_script_hash: get_cost!("assembly_script_hash"),
            assembly_script_hash_sha256: get_cost!("assembly_script_hash_sha256"),
            assembly_script_is_address_eoa: get_cost!("assembly_script_is_address_eoa"),
            assembly_script_keccak256_hash: get_cost!("assembly_script_keccak256_hash"),
            assembly_script_local_call: get_cost!("assembly_script_local_call"),
            assembly_script_local_execution: get_cost!("assembly_script_local_execution"),
            assembly_script_print: get_cost!("assembly_script_print"),
            assembly_script_seed: get_cost!("assembly_script_seed"),
            assembly_script_send_message: get_cost!("assembly_script_send_message"),
            assembly_script_set_bytecode: get_cost!("assembly_script_set_bytecode"),
            assembly_script_set_bytecode_for: get_cost!("assembly_script_set_bytecode_for"),
            assembly_script_set_data: get_cost!("assembly_script_set_data"),
            assembly_script_set_data_for: get_cost!("assembly_script_set_data_for"),
            assembly_script_signature_verify: get_cost!("assembly_script_signature_verify"),
            assembly_script_trace: get_cost!("assembly_script_trace"),
            assembly_script_transfer_coins: get_cost!("assembly_script_transfer_coins"),
            assembly_script_transfer_coins_for: get_cost!("assembly_script_transfer_coins_for"),
            assembly_script_unsafe_random: get_cost!("assembly_script_unsafe_random"),
            assembly_script_validate_address: get_cost!("assembly_script_validate_address"),

            // WasmV1 ABI costs
            abi_abort: get_cost!("abi_abort"),
            abi_add_native_amount: get_cost!("abi_add_native_amount"),
            abi_address_from_public_key: get_cost!("abi_address_from_public_key"),
            abi_append_ds_value: get_cost!("abi_append_ds_value"),
            abi_base58_check_to_bytes: get_cost!("abi_base58_check_to_bytes"),
            abi_bytes_to_base58_check: get_cost!("abi_bytes_to_base58_check"),
            abi_call: get_cost!("abi_call"),
            abi_caller_has_write_access: get_cost!("abi_caller_has_write_access"),
            abi_chain_id: get_cost!("abi_chain_id"),
            abi_check_address: get_cost!("abi_check_address"),
            abi_check_native_amount: get_cost!("abi_check_native_amount"),
            abi_check_pubkey: get_cost!("abi_check_pubkey"),
            abi_check_signature: get_cost!("abi_check_signature"),
            abi_checked_add_native_time: get_cost!("abi_checked_add_native_time"),
            abi_checked_div_native_time: get_cost!("abi_checked_div_native_time"),
            abi_checked_mul_native_time: get_cost!("abi_checked_mul_native_time"),
            abi_checked_scalar_div_native_time: get_cost!("abi_checked_scalar_div_native_time"),
            abi_checked_sub_native_time: get_cost!("abi_checked_sub_native_time"),
            abi_compare_address: get_cost!("abi_compare_address"),
            abi_compare_native_amount: get_cost!("abi_compare_native_amount"),
            abi_compare_native_time: get_cost!("abi_compare_native_time"),
            abi_compare_pub_key: get_cost!("abi_compare_pub_key"),
            abi_create_sc: get_cost!("abi_create_sc"),
            abi_deferred_call_cancel: get_cost!("abi_deferred_call_cancel"),
            abi_deferred_call_exists: get_cost!("abi_deferred_call_exists"),
            abi_deferred_call_register: get_cost!("abi_deferred_call_register"),
            abi_delete_ds_entry: get_cost!("abi_delete_ds_entry"),
            abi_div_rem_native_amount: get_cost!("abi_div_rem_native_amount"),
            abi_ds_entry_exists: get_cost!("abi_ds_entry_exists"),
            abi_evm_get_address_from_pubkey: get_cost!("abi_evm_get_address_from_pubkey"),
            abi_evm_get_pubkey_from_signature: get_cost!("abi_evm_get_pubkey_from_signature"),
            abi_evm_verify_signature: get_cost!("abi_evm_verify_signature"),
            abi_function_exists: get_cost!("abi_function_exists"),
            abi_generate_event: get_cost!("abi_generate_event"),
            abi_get_address_category: get_cost!("abi_get_address_category"),
            abi_get_address_version: get_cost!("abi_get_address_version"),
            abi_get_balance: get_cost!("abi_get_balance"),
            abi_get_bytecode: get_cost!("abi_get_bytecode"),
            abi_get_call_coins: get_cost!("abi_get_call_coins"),
            abi_get_call_stack: get_cost!("abi_get_call_stack"),
            abi_get_current_slot: get_cost!("abi_get_current_slot"),
            abi_get_deferred_call_quote: get_cost!("abi_get_deferred_call_quote"),
            abi_get_ds_keys: get_cost!("abi_get_ds_keys"),
            abi_get_ds_value: get_cost!("abi_get_ds_value"),
            abi_get_native_time: get_cost!("abi_get_native_time"),
            abi_get_op_data: get_cost!("abi_get_op_data"),
            abi_get_op_keys: get_cost!("abi_get_op_keys"),
            abi_get_origin_operation_id: get_cost!("abi_get_origin_operation_id"),
            abi_get_owned_addresses: get_cost!("abi_get_owned_addresses"),
            abi_get_pubkey_version: get_cost!("abi_get_pubkey_version"),
            abi_get_remaining_gas: get_cost!("abi_get_remaining_gas"),
            abi_get_signature_version: get_cost!("abi_get_signature_version"),
            abi_hash_blake3: get_cost!("abi_hash_blake3"),
            abi_hash_keccak256: get_cost!("abi_hash_keccak256"),
            abi_hash_sha256: get_cost!("abi_hash_sha256"),
            abi_is_address_eoa: get_cost!("abi_is_address_eoa"),
            abi_local_call: get_cost!("abi_local_call"),
            abi_local_execution: get_cost!("abi_local_execution"),
            abi_native_amount_from_string: get_cost!("abi_native_amount_from_string"),
            abi_native_amount_to_string: get_cost!("abi_native_amount_to_string"),
            abi_op_entry_exists: get_cost!("abi_op_entry_exists"),
            abi_scalar_div_rem_native_amount: get_cost!("abi_scalar_div_rem_native_amount"),
            abi_scalar_mul_native_amount: get_cost!("abi_scalar_mul_native_amount"),
            abi_send_async_message: get_cost!("abi_send_async_message"),
            abi_set_bytecode: get_cost!("abi_set_bytecode"),
            abi_set_ds_value: get_cost!("abi_set_ds_value"),
            abi_sub_native_amount: get_cost!("abi_sub_native_amount"),
            abi_transfer_coins: get_cost!("abi_transfer_coins"),
            abi_unsafe_random: get_cost!("abi_unsafe_random"),
            abi_verify_signature: get_cost!("abi_verify_signature"),
        })
    }
}

#[cfg(any(test, feature = "gas_calibration", feature = "testing"))]
impl Default for GasCosts {
    fn default() -> Self {
        Self {
            // Core costs
            launch_cost: 15702,
            operator_cost: 23,
            cl_compilation_cost: 745000000,
            sp_compilation_cost: 314000000,
            max_instance_cost: 2100000,

            // AssemblyScript ABI costs
            assembly_script_abort: 1,
            assembly_script_address_from_public_key: 1570,
            assembly_script_append_data: 1060,
            assembly_script_append_data_for: 1290,
            assembly_script_call: 15000,
            assembly_script_caller_has_write_access: 775,
            assembly_script_chain_id: 785,
            assembly_script_console_debug: 855,
            assembly_script_console_error: 855,
            assembly_script_console_info: 855,
            assembly_script_console_log: 855,
            assembly_script_console_warn: 855,
            assembly_script_create_sc: 745000000,
            assembly_script_date_now: 355,
            assembly_script_deferred_call_cancel: 4165,
            assembly_script_deferred_call_exists: 6580,
            assembly_script_deferred_call_register: 2650,
            assembly_script_delete_data: 980,
            assembly_script_delete_data_for: 1100,
            assembly_script_evm_get_address_from_pubkey: 1120,
            assembly_script_evm_get_pubkey_from_signature: 1540,
            assembly_script_evm_signature_verify: 3310,
            assembly_script_function_exists: 2875,
            assembly_script_generate_event: 860,
            assembly_script_get_balance: 745,
            assembly_script_get_balance_for: 900,
            assembly_script_get_bytecode: 1100,
            assembly_script_get_bytecode_for: 1375,
            assembly_script_get_call_coins: 725,
            assembly_script_get_call_stack: 1560,
            assembly_script_get_current_period: 785,
            assembly_script_get_current_thread: 770,
            assembly_script_get_data: 1040,
            assembly_script_get_data_for: 1240,
            assembly_script_get_deferred_call_quote: 1220,
            assembly_script_get_keys: 1000,
            assembly_script_get_keys_for: 1195,
            assembly_script_get_op_data: 50000,
            assembly_script_get_op_keys: 1400,
            assembly_script_get_op_keys_prefix: 1400,
            assembly_script_get_origin_operation_id: 785,
            assembly_script_get_owned_addresses: 1600,
            assembly_script_get_remaining_gas: 580,
            assembly_script_get_time: 750,
            assembly_script_has_data: 845,
            assembly_script_has_data_for: 1220,
            assembly_script_has_op_key: 1455,
            assembly_script_hash: 1055,
            assembly_script_hash_sha256: 990,
            assembly_script_is_address_eoa: 450,
            assembly_script_keccak256_hash: 1055,
            assembly_script_local_call: 15000,
            assembly_script_local_execution: 314000000,
            assembly_script_print: 855,
            assembly_script_seed: 360,
            assembly_script_send_message: 40000000,
            assembly_script_set_bytecode: 745000000,
            assembly_script_set_bytecode_for: 745000000,
            assembly_script_set_data: 940,
            assembly_script_set_data_for: 1070,
            assembly_script_signature_verify: 1200,
            assembly_script_trace: 855,
            assembly_script_transfer_coins: 1045,
            assembly_script_transfer_coins_for: 1190,
            assembly_script_unsafe_random: 790,
            assembly_script_validate_address: 890,

            // WasmV1 ABI costs
            abi_abort: 0,
            abi_add_native_amount: 2415,
            abi_address_from_public_key: 2410,
            abi_append_ds_value: 2130,
            abi_base58_check_to_bytes: 13380,
            abi_bytes_to_base58_check: 31550,
            abi_call: 85637,
            abi_caller_has_write_access: 1360,
            abi_chain_id: 1505,
            abi_check_address: 1800,
            abi_check_native_amount: 1765,
            abi_check_pubkey: 1880,
            abi_check_signature: 1985,
            abi_checked_add_native_time: 2060,
            abi_checked_div_native_time: 2310,
            abi_checked_mul_native_time: 1750,
            abi_checked_scalar_div_native_time: 2200,
            abi_checked_sub_native_time: 2065,
            abi_compare_address: 2050,
            abi_compare_native_amount: 1955,
            abi_compare_native_time: 2130,
            abi_compare_pub_key: 2660,
            abi_create_sc: 2395,
            abi_deferred_call_cancel: 3750,
            abi_deferred_call_exists: 2215,
            abi_deferred_call_register: 3725,
            abi_delete_ds_entry: 1115,
            abi_div_rem_native_amount: 2315,
            abi_ds_entry_exists: 1870,
            abi_evm_get_address_from_pubkey: 2180,
            abi_evm_get_pubkey_from_signature: 16275,
            abi_evm_verify_signature: 14195,
            abi_function_exists: 2380,
            abi_generate_event: 2105,
            abi_get_address_category: 1800,
            abi_get_address_version: 1695,
            abi_get_balance: 1765,
            abi_get_bytecode: 1465,
            abi_get_call_coins: 1590,
            abi_get_call_stack: 2015,
            abi_get_current_slot: 1505,
            abi_get_deferred_call_quote: 2080,
            abi_get_ds_keys: 1825,
            abi_get_ds_value: 2440,
            abi_get_native_time: 1625,
            abi_get_op_data: 1805,
            abi_get_op_keys: 23185,
            abi_get_origin_operation_id: 1310,
            abi_get_owned_addresses: 2035,
            abi_get_pubkey_version: 1920,
            abi_get_remaining_gas: 1455,
            abi_get_signature_version: 1755,
            abi_hash_blake3: 2705,
            abi_hash_keccak256: 2620,
            abi_hash_sha256: 2725,
            abi_is_address_eoa: 1720,
            abi_local_call: 83348,
            abi_local_execution: 86310,
            abi_native_amount_from_string: 1615,
            abi_native_amount_to_string: 1880,
            abi_op_entry_exists: 1860,
            abi_scalar_div_rem_native_amount: 2320,
            abi_scalar_mul_native_amount: 2200,
            abi_send_async_message: 40000000,
            abi_set_bytecode: 238,
            abi_set_ds_value: 2020,
            abi_sub_native_amount: 2130,
            abi_transfer_coins: 2250,
            abi_unsafe_random: 2010,
            abi_verify_signature: 5960,
        }
    }
}

#[allow(unused_variables)]
pub trait Interface: Send + Sync + InterfaceClone {
    fn increment_recursion_counter(&self) -> Result<()>;

    fn decrement_recursion_counter(&self) -> Result<()>;

    fn get_interface_version(&self) -> Result<u32>;

    /// Prepare the execution of a module at the given address and transfer a
    /// given amount of coins
    fn init_call(&self, address: &str, raw_coins: u64) -> Result<Vec<u8>>;

    /// Prepare the execution of a module at the given address and transfer a
    /// given amount of coins

    fn init_call_wasmv1(&self, address: &str, raw_coins: NativeAmount) -> Result<Vec<u8>>;

    /// Finish a call
    fn finish_call(&self) -> Result<()>;

    /// Get the SCE ledger balance for the current address.
    /// Defaults to zero if the address is not found.
    fn get_balance(&self) -> Result<u64>;

    /// Get the SCE ledger balance for an address.
    /// Defaults to zero if the address is not found.
    fn get_balance_for(&self, address: &str) -> Result<u64>;

    fn get_balance_wasmv1(&self, address: Option<String>) -> Result<NativeAmount>;

    /// Transfer an amount from the address on the current call stack to a
    /// target address.
    fn transfer_coins(&self, to_address: &str, raw_amount: u64) -> Result<()>;

    /// Transfer an amount from the specified address to a target address.
    fn transfer_coins_for(
        &self,
        from_address: &str,
        to_address: &str,
        raw_amount: u64,
    ) -> Result<()>;

    fn transfer_coins_wasmv1(
        &self,
        to_address: String,
        raw_amount: NativeAmount,
        from_address: Option<String>,
    ) -> Result<()>;

    /// Get the amount of coins that have been made available for use by the
    /// caller of the currently executing code.
    fn get_call_coins(&self) -> Result<u64> {
        Err(InterfaceError::GenericError(
            "unimplemented function get_call_coins_for in interface".into(),
        ))
    }

    /// Get the native amount of coins that have been made available for use by
    /// the caller of the currently executing code.
    fn get_call_coins_wasmv1(&self) -> Result<NativeAmount>;

    /// Sets the executable bytecode at a current address.
    fn raw_set_bytecode(&self, bytecode: &[u8]) -> Result<()>;

    /// Sets the executable bytecode at a target address.
    /// The target address must exist and the current context must have access
    /// rights.
    fn raw_set_bytecode_for(&self, address: &str, bytecode: &[u8]) -> Result<()>;

    fn set_bytecode_wasmv1(&self, bytecode: &[u8], address: Option<String>) -> Result<()>;

    /// Requires a new address that contains the sent &[u8]
    fn create_module(&self, module: &[u8]) -> Result<String>;

    /// Print function for examples
    fn print(&self, message: &str) -> Result<()>;

    /// Return datastore keys
    /// Will only return keys with a given prefix if provided in args
    fn get_keys(&self, prefix: Option<&[u8]>) -> Result<BTreeSet<Vec<u8>>>;

    /// Return datastore keys
    /// Will only return keys with a given prefix if provided in args
    fn get_keys_for(&self, address: &str, prefix: Option<&[u8]>) -> Result<BTreeSet<Vec<u8>>>;

    fn get_ds_keys_wasmv1(
        &self,
        prefix: &[u8],
        address: Option<String>,
    ) -> Result<BTreeSet<Vec<u8>>>;

    /// Return the datastore value of the corresponding key
    fn raw_get_data(&self, key: &[u8]) -> Result<Vec<u8>>;

    /// Requires the data at the address
    fn raw_get_data_for(&self, address: &str, key: &[u8]) -> Result<Vec<u8>>;

    fn get_ds_value_wasmv1(&self, key: &[u8], address: Option<String>) -> Result<Vec<u8>>;

    /// Set the datastore value for the corresponding key
    fn raw_set_data(&self, key: &[u8], value: &[u8]) -> Result<()>;

    /// Set the datastore value for the corresponding key of the given address
    fn raw_set_data_for(&self, address: &str, key: &[u8], value: &[u8]) -> Result<()>;

    fn set_ds_value_wasmv1(&self, key: &[u8], value: &[u8], address: Option<String>) -> Result<()>;

    /// Append a value to the current datastore value for the corresponding key
    fn raw_append_data(&self, key: &[u8], value: &[u8]) -> Result<()>;

    /// Append a value to the current datastore value for the corresponding key
    /// and the given address
    fn raw_append_data_for(&self, address: &str, key: &[u8], value: &[u8]) -> Result<()>;

    fn append_ds_value_wasmv1(
        &self,
        key: &[u8],
        value: &[u8],
        address: Option<String>,
    ) -> Result<()>;

    /// Delete a datastore entry
    fn raw_delete_data(&self, key: &[u8]) -> Result<()>;

    /// Delete a datastore entry at of the given address
    fn raw_delete_data_for(&self, address: &str, key: &[u8]) -> Result<()>;

    fn delete_ds_entry_wasmv1(&self, key: &[u8], address: Option<String>) -> Result<()>;

    /// Requires to replace the data in the current address
    ///
    /// Note:
    /// The execution lib will always use the current context address for the
    /// update
    fn has_data(&self, key: &[u8]) -> Result<bool>;

    /// Check if a datastore entry exists
    fn has_data_for(&self, address: &str, key: &[u8]) -> Result<bool>;

    fn ds_entry_exists_wasmv1(&self, key: &[u8], address: Option<String>) -> Result<bool>;

    /// Returns bytecode of the current address
    fn raw_get_bytecode(&self) -> Result<Vec<u8>>;

    /// Returns bytecode of the target address
    fn raw_get_bytecode_for(&self, address: &str) -> Result<Vec<u8>>;

    fn get_bytecode_wasmv1(&self, address: Option<String>) -> Result<Vec<u8>>;

    /// Return operation datastore keys
    fn get_op_keys(&self, prefix: Option<&[u8]>) -> Result<Vec<Vec<u8>>>;

    fn get_op_keys_wasmv1(&self, prefix: &[u8]) -> Result<Vec<Vec<u8>>>;

    /// Check if operation in datastore exists
    fn op_entry_exists(&self, key: &[u8]) -> Result<bool>;

    /// Return operation datastore data for a given key
    fn get_op_data(&self, key: &[u8]) -> Result<Vec<u8>>;

    /// Check whether or not the caller has write access in the current context
    fn caller_has_write_access(&self) -> Result<bool>;

    /// Hash data
    fn hash(&self, data: &[u8]) -> Result<[u8; 32]>;

    /// Returns the blake3 hash of the given bytes
    fn hash_blake3(&self, bytes: &[u8]) -> Result<[u8; 32]>;

    /// Verify signature
    fn signature_verify(&self, data: &[u8], signature: &str, public_key: &str) -> Result<bool>;

    /// Verify signature (EVM)
    fn evm_signature_verify(
        &self,
        message: &[u8],
        signature: &[u8],
        public_key: &[u8],
    ) -> Result<bool>;

    /// Get address from public key (EVM)
    fn evm_get_address_from_pubkey(&self, public_key: &[u8]) -> Result<Vec<u8>>;

    /// Get public key from signature (EVM)
    fn evm_get_pubkey_from_signature(&self, hash: &[u8], signature: &[u8]) -> Result<Vec<u8>>;

    /// Return true if the address is a User address, false if it is an SC
    /// address
    fn is_address_eoa(&self, address: &str) -> Result<bool>;

    /// Convert a public key to an address
    fn address_from_public_key(&self, public_key: &str) -> Result<String>;

    /// Validate an address
    fn validate_address(&self, address: &str) -> Result<bool>;

    /// Returns the current time (millisecond unix timestamp)
    fn get_time(&self) -> Result<u64>;

    /// Returns a random number (unsafe: can be predicted and manipulated)
    fn unsafe_random(&self) -> Result<i64>;

    /// Returns a random number (unsafe: can be predicted and manipulated)
    fn unsafe_random_f64(&self) -> Result<f64>;

    /// Returns a random number (unsafe: can be predicted and manipulated)
    fn unsafe_random_wasmv1(&self, num_bytes: u64) -> Result<Vec<u8>>;

    /// Returns the period of the current execution slot
    fn get_current_period(&self) -> Result<u64>;

    /// Returns the thread of the current execution slot
    fn get_current_thread(&self) -> Result<u8>;

    /// Returns the current execution slot
    fn get_current_slot(&self) -> Result<Slot>;

    /// Expect to return a list of owned addresses
    ///
    /// Required on smart-contract execute the imported function
    /// `assembly_script_get_owned_addresses`
    fn get_owned_addresses(&self) -> Result<Vec<String>>;

    /// Expect to return a list of addresses in the call stack
    ///
    /// Required on smart-contract execute the imported function
    /// `assembly_script_get_call_stack`
    fn get_call_stack(&self) -> Result<Vec<String>>;

    /// Generate a smart contract event
    fn generate_event(&self, _event: String) -> Result<()>;

    /// Generate a smart contract event
    fn generate_event_wasmv1(&self, _event: Vec<u8>) -> Result<()>;

    /// For the given bytecode:
    ///
    /// * Get the corresponding runtime module if it already exists
    /// * Compile it if not
    ///
    /// Returns a CL compiled module and the remaining gas after loading
    fn get_module(&self, bytecode: &[u8], gas_limit: u64) -> Result<RuntimeModule>;

    /// Compile a temportary module from the given bytecode
    ///
    /// Returns a SP compiled module and the remaining gas after loading
    fn get_tmp_module(&self, bytecode: &[u8], gas_limit: u64) -> Result<RuntimeModule>;

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
    ) -> Result<()>;

    // Returns the operation id that originated the current execution if there
    // is one
    fn get_origin_operation_id(&self) -> Result<Option<String>>;

    // Sha256 hash bytes
    fn hash_sha256(&self, bytes: &[u8]) -> Result<[u8; 32]>;

    // Keccak256 hash bytes
    fn hash_keccak256(&self, bytes: &[u8]) -> Result<[u8; 32]>;

    // Return the current chain id
    fn chain_id(&self) -> Result<u64>;

    // Return a boolean that determine if there is place in this slot and an amount of fee needed to take the space
    fn get_deferred_call_quote(
        &self,
        target_slot: (u64, u8),
        gas_limit: u64,
        params_size: u64,
    ) -> Result<(bool, u64)>;

    // Register a new deferred call and return his id
    fn deferred_call_register(
        &self,
        target_addr: &str,
        target_func: &str,
        target_slot: (u64, u8),
        max_gas: u64,
        params: &[u8],
        coins: u64,
    ) -> Result<String>;

    // Return true if the current deferred call exists
    fn deferred_call_exists(&self, id: &str) -> Result<bool>;

    // Cancel a deferred call (will return the coins)
    fn deferred_call_cancel(&self, id: &str) -> Result<()>;

    fn native_amount_from_str_wasmv1(&self, amount: &str) -> Result<NativeAmount>;

    fn native_amount_to_string_wasmv1(&self, amount: &NativeAmount) -> Result<String>;

    fn check_native_amount_wasmv1(&self, amount: &NativeAmount) -> Result<bool>;

    fn add_native_amount_wasmv1(
        &self,
        amount1: &NativeAmount,
        amount2: &NativeAmount,
    ) -> Result<NativeAmount>;

    fn sub_native_amount_wasmv1(
        &self,
        amount1: &NativeAmount,
        amount2: &NativeAmount,
    ) -> Result<NativeAmount>;

    fn scalar_mul_native_amount_wasmv1(
        &self,
        amount: &NativeAmount,
        factor: u64,
    ) -> Result<NativeAmount>;

    fn scalar_div_rem_native_amount_wasmv1(
        &self,
        dividend: &NativeAmount,
        divisor: u64,
    ) -> Result<(NativeAmount, NativeAmount)>;

    fn div_rem_native_amount_wasmv1(
        &self,
        dividend: &NativeAmount,
        divisor: &NativeAmount,
    ) -> Result<(u64, NativeAmount)>;

    fn check_address_wasmv1(&self, to_check: &str) -> Result<bool>;

    fn check_pubkey_wasmv1(&self, to_check: &str) -> Result<bool>;

    fn check_signature_wasmv1(&self, to_check: &str) -> Result<bool>;

    fn get_address_category_wasmv1(&self, to_check: &str) -> Result<AddressCategory>;

    fn get_address_version_wasmv1(&self, address: &str) -> Result<u64>;

    fn get_pubkey_version_wasmv1(&self, pubkey: &str) -> Result<u64>;

    fn get_signature_version_wasmv1(&self, signature: &str) -> Result<u64>;

    fn checked_add_native_time_wasmv1(
        &self,
        time1: &NativeTime,
        time2: &NativeTime,
    ) -> Result<NativeTime>;

    fn checked_sub_native_time_wasmv1(
        &self,
        time1: &NativeTime,
        time2: &NativeTime,
    ) -> Result<NativeTime>;

    fn checked_mul_native_time_wasmv1(&self, time: &NativeTime, factor: u64) -> Result<NativeTime>;

    fn checked_scalar_div_native_time_wasmv1(
        &self,
        dividend: &NativeTime,
        divisor: u64,
    ) -> Result<(NativeTime, NativeTime)>;

    fn checked_div_native_time_wasmv1(
        &self,
        dividend: &NativeTime,
        divisor: &NativeTime,
    ) -> Result<(u64, NativeTime)>;

    fn base58_check_to_bytes_wasmv1(&self, s: &str) -> Result<Vec<u8>>;

    fn bytes_to_base58_check_wasmv1(&self, bytes: &[u8]) -> String;

    fn compare_address_wasmv1(&self, left: &str, right: &str) -> Result<ComparisonResult>;

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

    fn compare_pub_key_wasmv1(&self, left: &str, right: &str) -> Result<ComparisonResult>;

    fn save_gas_remaining_before_subexecution(&self, gas_used_until: u64);
}

impl dyn Interface {
    pub fn get_data<T: DeserializeOwned>(&self, key: &[u8]) -> Result<T> {
        Ok(serde_json::from_str::<T>(std::str::from_utf8(
            &self.raw_get_data(key)?,
        )?)?)
    }

    pub fn get_data_for<T: DeserializeOwned>(&self, address: &str, key: &[u8]) -> Result<T> {
        Ok(serde_json::from_str::<T>(std::str::from_utf8(
            &self.raw_get_data_for(address, key)?,
        )?)?)
    }

    pub fn set_data<T: Serialize>(&self, key: &[u8], value: &T) -> Result<()> {
        // TODO: Avoid using this many conversions, protobuf serialization
        // should be enough
        self.raw_set_data(key, serde_json::to_string::<T>(value)?.as_bytes())
    }

    pub fn set_data_for<T: Serialize>(&self, address: &str, key: &[u8], value: &T) -> Result<()> {
        self.raw_set_data_for(address, key, serde_json::to_string::<T>(value)?.as_bytes())
    }
}
