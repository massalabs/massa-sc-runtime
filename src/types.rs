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
    pub(crate) abi_costs: HashMap<String, u64>,
    pub(crate) launch_cost: u64,
    pub(crate) operator_cost: u64,
    pub cl_compilation_cost: u64,
    pub sp_compilation_cost: u64,
    pub max_instance_cost: u64,
}

impl GasCosts {
    pub fn new(abi_cost_file: PathBuf) -> Result<Self> {
        let abi_cost_file = std::fs::read_to_string(abi_cost_file)?;
        let mut abi_costs: HashMap<String, u64> = serde_json::from_str(&abi_cost_file)?;
        abi_costs.iter_mut().for_each(|(_, v)| {
            let unit_digit = *v % 10;
            if unit_digit > 5 {
                *v += 10 - unit_digit;
            } else {
                *v -= unit_digit;
            }
        });
        Ok(Self {
            // Note: Use a constant = 23 here in order to not break compatibility with previous Massa node version
            //       The gas calibration for wasm operators is very incomplete for now and should be reworked
            //       See: https://github.com/massalabs/gas-calibration/issues/9
            operator_cost: 23,
            launch_cost: *abi_costs.get("launch").ok_or_else(|| {
                InterfaceError::GasCalibrationError(
                    "launch cost not found in ABI gas cost file.".into(),
                )
            })?,
            cl_compilation_cost: *abi_costs.get("cl_compilation").ok_or_else(|| {
                InterfaceError::GasCalibrationError(
                    "cl_compilation cost not found in ABI gas cost file.".into(),
                )
            })?,
            sp_compilation_cost: *abi_costs.get("sp_compilation").ok_or_else(|| {
                InterfaceError::GasCalibrationError(
                    "sp_compilation cost not found in ABI gas cost file.".into(),
                )
            })?,
            max_instance_cost: *abi_costs.get("max_instance").ok_or_else(|| {
                InterfaceError::GasCalibrationError(
                    "max_instance cost not found in ABI gas cost file.".into(),
                )
            })?,
            abi_costs,
        })
    }

    #[cfg(any(test, feature = "gas_calibration", feature = "testing"))]
    pub fn get_abi_costs(&self) -> &HashMap<String, u64> {
        &self.abi_costs
    }

    #[cfg(any(test, feature = "gas_calibration", feature = "testing"))]
    /// Insert into hashmap - return None if already there
    fn try_insert(h: &mut HashMap<String, u64>, k: String, v: u64) {
        // Note: use try_insert when not an experimental feature anymore
        // https://doc.rust-lang.org/std/collections/struct.HashMap.html#method.try_insert
        if let Some(_) = h.insert(k.clone(), v) {
            // Insert return old value if the key was already there
            panic!("Key {} already in hashmap", k);
        }
    }
}

#[cfg(any(test, feature = "gas_calibration", feature = "testing"))]
impl Default for GasCosts {
    fn default() -> Self {
        let mut h = HashMap::new();
        // Note: use try_insert to notify devs for duplicated abi/gas_costs
        //       this will panic in unit tests
        GasCosts::try_insert(
            &mut h,
            String::from("assembly_script_address_from_public_key"),
            147,
        );
        GasCosts::try_insert(&mut h, String::from("assembly_script_validate_address"), 4);
        GasCosts::try_insert(&mut h, String::from("assembly_script_append_data"), 162);
        GasCosts::try_insert(&mut h, String::from("assembly_script_append_data_for"), 200);
        GasCosts::try_insert(&mut h, String::from("assembly_script_create_sc"), 160);
        GasCosts::try_insert(&mut h, String::from("assembly_script_delete_data"), 78);
        GasCosts::try_insert(&mut h, String::from("assembly_script_delete_data_for"), 120);
        GasCosts::try_insert(&mut h, String::from("assembly_script_generate_event"), 36);
        GasCosts::try_insert(&mut h, String::from("assembly_script_get_balance"), 4);
        GasCosts::try_insert(&mut h, String::from("assembly_script_get_balance_for"), 41);
        GasCosts::try_insert(&mut h, String::from("assembly_script_get_call_coins"), 9);
        GasCosts::try_insert(&mut h, String::from("assembly_script_get_call_stack"), 56);
        GasCosts::try_insert(&mut h, String::from("assembly_script_get_data"), 85);
        GasCosts::try_insert(&mut h, String::from("assembly_script_get_data_for"), 139);
        GasCosts::try_insert(&mut h, String::from("assembly_script_get_keys"), 26);
        GasCosts::try_insert(&mut h, String::from("assembly_script_get_keys_for"), 48);
        GasCosts::try_insert(&mut h, String::from("assembly_script_get_op_data"), 71);
        GasCosts::try_insert(&mut h, String::from("assembly_script_get_op_keys"), 138);
        GasCosts::try_insert(
            &mut h,
            String::from("assembly_script_get_op_keys_prefix"),
            138,
        );
        GasCosts::try_insert(
            &mut h,
            String::from("assembly_script_get_owned_addresses"),
            52,
        );
        GasCosts::try_insert(&mut h, String::from("assembly_script_get_remaining_gas"), 7);
        GasCosts::try_insert(&mut h, String::from("assembly_script_get_time"), 4);
        GasCosts::try_insert(&mut h, String::from("assembly_script_has_data"), 69);
        GasCosts::try_insert(&mut h, String::from("assembly_script_has_data_for"), 115);
        GasCosts::try_insert(&mut h, String::from("assembly_script_has_op_key"), 78);
        GasCosts::try_insert(&mut h, String::from("assembly_script_hash"), 83);
        GasCosts::try_insert(&mut h, String::from("assembly_script_hash_sha256"), 83);
        GasCosts::try_insert(&mut h, String::from("assembly_script_keccak256_hash"), 83);
        GasCosts::try_insert(&mut h, String::from("assembly_script_print"), 35);
        GasCosts::try_insert(&mut h, String::from("assembly_script_send_message"), 316);
        GasCosts::try_insert(
            &mut h,
            String::from("assembly_script_get_origin_operation_id"),
            200,
        );
        GasCosts::try_insert(&mut h, String::from("assembly_script_set_bytecode"), 74);
        GasCosts::try_insert(
            &mut h,
            String::from("assembly_script_set_bytecode_for"),
            129,
        );
        GasCosts::try_insert(&mut h, String::from("assembly_script_set_data"), 158);
        GasCosts::try_insert(&mut h, String::from("assembly_script_set_data_for"), 165);
        GasCosts::try_insert(&mut h, String::from("assembly_script_signature_verify"), 98);
        GasCosts::try_insert(
            &mut h,
            String::from("assembly_script_evm_signature_verify"),
            264,
        );
        GasCosts::try_insert(
            &mut h,
            String::from("assembly_script_evm_get_address_from_pubkey"),
            11,
        );
        GasCosts::try_insert(
            &mut h,
            String::from("assembly_script_evm_get_pubkey_from_signature"),
            11,
        );
        GasCosts::try_insert(&mut h, String::from("assembly_script_is_address_eoa"), 11);
        GasCosts::try_insert(&mut h, String::from("assembly_script_transfer_coins"), 62);
        GasCosts::try_insert(
            &mut h,
            String::from("assembly_script_transfer_coins_for"),
            102,
        );
        GasCosts::try_insert(&mut h, String::from("assembly_script_unsafe_random"), 11);
        GasCosts::try_insert(&mut h, String::from("assembly_script_call"), 11);
        GasCosts::try_insert(&mut h, String::from("assembly_script_local_call"), 11);
        GasCosts::try_insert(&mut h, String::from("assembly_script_local_execution"), 11);
        GasCosts::try_insert(&mut h, String::from("assembly_script_get_bytecode"), 11);
        GasCosts::try_insert(&mut h, String::from("assembly_script_get_bytecode_for"), 11);
        GasCosts::try_insert(
            &mut h,
            String::from("assembly_script_caller_has_write_access"),
            11,
        );
        GasCosts::try_insert(&mut h, String::from("assembly_script_function_exists"), 11);
        GasCosts::try_insert(
            &mut h,
            String::from("assembly_script_get_deferred_call_quote"),
            244,
        );
        GasCosts::try_insert(
            &mut h,
            String::from("assembly_script_deferred_call_register"),
            530,
        );
        GasCosts::try_insert(
            &mut h,
            String::from("assembly_script_deferred_call_exists"),
            1316,
        );
        GasCosts::try_insert(
            &mut h,
            String::from("assembly_script_deferred_call_cancel"),
            833,
        );
        GasCosts::try_insert(&mut h, String::from("assembly_script_seed"), 11);
        GasCosts::try_insert(&mut h, String::from("assembly_script_abort"), 11);
        GasCosts::try_insert(&mut h, String::from("assembly_script_date_now"), 11);
        GasCosts::try_insert(&mut h, String::from("assembly_script_console_log"), 36); // same cost as for generate_event
        GasCosts::try_insert(&mut h, String::from("assembly_script_console_info"), 36);
        GasCosts::try_insert(&mut h, String::from("assembly_script_console_debug"), 36);
        GasCosts::try_insert(&mut h, String::from("assembly_script_console_warn"), 36);
        GasCosts::try_insert(&mut h, String::from("assembly_script_console_error"), 36);
        GasCosts::try_insert(&mut h, String::from("assembly_script_trace"), 36);
        GasCosts::try_insert(&mut h, String::from("assembly_script_chain_id"), 9);
        GasCosts::try_insert(
            &mut h,
            String::from("assembly_script_get_current_period"),
            157,
        );
        GasCosts::try_insert(
            &mut h,
            String::from("assembly_script_get_current_thread"),
            154,
        );

        // abi v1
        GasCosts::try_insert(&mut h, String::from("abi_get_address_version"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_call"), 30);
        GasCosts::try_insert(
            &mut h,
            String::from("abi_checked_scalar_div_native_time"),
            30,
        );
        GasCosts::try_insert(&mut h, String::from("abi_check_signature"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_local_call"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_compare_address"), 30);
        GasCosts::try_insert(&mut h, String::from("sp_compilation"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_append_ds_value"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_delete_ds_entry"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_address_from_public_key"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_bytes_to_base58_check"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_hash_blake3"), 30);
        GasCosts::try_insert(
            &mut h,
            String::from("abi_evm_get_pubkey_from_signature"),
            30,
        );
        GasCosts::try_insert(&mut h, String::from("abi_get_remaining_gas"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_add_native_amount"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_base58_check_to_bytes"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_set_bytecode"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_get_call_coins"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_chain_id"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_abort"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_checked_div_native_time"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_get_address_category"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_send_async_message"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_check_address"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_sub_native_amount"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_scalar_mul_native_amount"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_get_bytecode"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_get_op_keys"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_native_amount_to_string"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_hash_sha256"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_native_amount_from_string"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_create_sc"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_get_origin_operation_id"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_get_pubkey_version"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_is_address_eoa"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_evm_get_address_from_pubkey"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_get_ds_value"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_scalar_div_rem_native_amount"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_get_signature_version"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_unsafe_random"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_compare_pub_key"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_generate_event"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_verify_signature"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_checked_sub_native_time"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_get_native_time"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_get_owned_addresses"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_check_pubkey"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_compare_native_amount"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_compare_native_time"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_get_balance"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_set_ds_value"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_get_ds_keys"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_get_op_data"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_ds_entry_exists"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_div_rem_native_amount"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_evm_verify_signature"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_check_native_amount"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_transfer_coins"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_hash_keccak256"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_get_current_slot"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_checked_add_native_time"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_op_entry_exists"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_function_exists"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_get_call_stack"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_caller_has_write_access"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_local_execution"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_checked_mul_native_time"), 30);
        GasCosts::try_insert(&mut h, String::from("abi_deferred_call_cancel"), 750);
        GasCosts::try_insert(&mut h, String::from("abi_deferred_call_exists"), 443);
        GasCosts::try_insert(&mut h, String::from("abi_deferred_call_register"), 745);
        GasCosts::try_insert(&mut h, String::from("abi_get_deferred_call_quote"), 416);

        Self {
            abi_costs: h,
            operator_cost: 1,
            launch_cost: 10_000,
            sp_compilation_cost: 314_000_000,
            cl_compilation_cost: 745_000_000,
            max_instance_cost: 2_100_000,
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
