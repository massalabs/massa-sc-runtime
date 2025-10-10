use super::super::{env::ABIEnv, WasmV1Error};
// handle_abi and handle_abi_raw are now macros exported at crate root
use massa_proto_rs::massa::{
    abi::v1::{self as proto, *},
    model::v1::NativeTime,
};
use wasmer::{imports, AsStoreMut, Function, FunctionEnv, FunctionEnvMut, Imports};

use crate::Interface;
#[cfg(feature = "execution-trace")]
use crate::{into_trace_value, AbiTrace, AbiTraceType};
#[cfg(feature = "execution-trace")]
use rust_decimal::prelude::ToPrimitive;
#[cfg(feature = "execution-trace")]
use rust_decimal::Decimal;

// This macro ease the construction of the Error variant of the response to an
// ABI call.
macro_rules! resp_err {
    ($err:expr) => {
        Ok(AbiResponse {
            resp: Some(abi_response::Resp::Error(proto::Error {
                message: $err.to_string(),
            })),
        })
    };
}

// Same as resp_err but for the ok variant of the response.
macro_rules! resp_ok {
    ($result:tt, { $($field:ident $(: $value:expr)?),* $(,)? }) => {
        Ok(AbiResponse {
            resp: Some(abi_response::Resp::Res(RespResult {
                res: Some(resp_result::Res::$result($result {
                    $($field $(: $value)?,)*
                }))
            }))
        })
    };
}

/// Register all ABIs to a store
pub fn register_abis(store: &mut impl AsStoreMut, shared_abi_env: ABIEnv) -> Imports {
    let fn_env = FunctionEnv::new(store, shared_abi_env);

    // helper macro to ease the construction of the imports
    macro_rules! abis {
        ($($name:expr => $func:ident),*) => {
            imports! {
                "massa" => {
                    $( $name => Function::new_typed_with_env(store, &fn_env, $func) ),*
                }
            }
        };
    }

    let imports = abis!(
        "abi_abort" => abi_abort,
        "abi_add_native_amount" => abi_add_native_amount,
        "abi_address_from_public_key" => abi_address_from_public_key,
        "abi_append_ds_value" => abi_append_ds_value,
        "abi_base58_check_to_bytes" => abi_base58_check_to_bytes,
        "abi_bytes_to_base58_check" => abi_bytes_to_base58_check,
        "abi_call" => abi_call,
        "abi_caller_has_write_access" => abi_caller_has_write_access,
        "abi_check_address" => abi_check_address,
        "abi_check_native_amount" => abi_check_native_amount,
        "abi_check_pubkey" => abi_check_pubkey,
        "abi_check_signature" => abi_check_signature,
        "abi_checked_add_native_time" => abi_checked_add_native_time,
        "abi_checked_div_native_time" => abi_checked_div_native_time,
        "abi_checked_mul_native_time" => abi_checked_mul_native_time,
        "abi_checked_scalar_div_native_time" => abi_checked_scalar_div_native_time,
        "abi_checked_sub_native_time" => abi_checked_sub_native_time,
        "abi_compare_address" => abi_compare_address,
        "abi_compare_native_amount" => abi_compare_native_amount,
        "abi_compare_native_time" => abi_compare_native_time,
        "abi_compare_pub_key" => abi_compare_pub_key,
        "abi_create_sc" => abi_create_sc,
        "abi_delete_ds_entry" => abi_delete_ds_entry,
        "abi_div_rem_native_amount" => abi_div_rem_native_amount,
        "abi_ds_entry_exists" => abi_ds_entry_exists,
        "abi_function_exists" => abi_function_exists,
        "abi_generate_event" => abi_generate_event,
        "abi_get_address_category" => abi_get_address_category,
        "abi_get_address_version" => abi_get_address_version,
        "abi_get_balance" => abi_get_balance,
        "abi_get_bytecode" => abi_get_bytecode,
        "abi_get_call_coins" => abi_get_call_coins,
        "abi_get_call_stack" => abi_get_call_stack,
        "abi_get_current_slot" => abi_get_current_slot,
        "abi_get_ds_keys" => abi_get_ds_keys,
        "abi_get_ds_value" => abi_get_ds_value,
        "abi_get_native_time" => abi_get_native_time,
        "abi_get_op_data" => abi_get_op_data,
        "abi_get_op_keys" => abi_get_op_keys,
        "abi_get_origin_operation_id" => abi_get_origin_operation_id,
        "abi_get_owned_addresses" => abi_get_owned_addresses,
        "abi_get_pubkey_version" => abi_get_pubkey_version,
        "abi_get_remaining_gas" => abi_get_remaining_gas,
        "abi_get_signature_version" => abi_get_signature_version,
        "abi_op_entry_exists" => abi_op_entry_exists,
        "abi_hash_blake3" => abi_hash_blake3,
        "abi_hash_keccak256" => abi_hash_keccak256,
        "abi_hash_sha256" => abi_hash_sha256,
        "abi_local_call" => abi_local_call,
        "abi_local_execution" => abi_local_execution,
        "abi_native_amount_from_string" => abi_native_amount_from_string,
        "abi_native_amount_to_string" => abi_native_amount_to_string,
        "abi_scalar_div_rem_native_amount" => abi_scalar_div_rem_native_amount,
        "abi_scalar_mul_native_amount" => abi_scalar_mul_native_amount,
        "abi_send_async_message" => abi_send_async_message,
        "abi_set_bytecode" => abi_set_bytecode,
        "abi_set_ds_value" => abi_set_ds_value,
        "abi_sub_native_amount" => abi_sub_native_amount,
        "abi_transfer_coins" => abi_transfer_coins,
        "abi_unsafe_random" => abi_unsafe_random,
        "abi_verify_signature" => abi_verify_signature,
        "abi_evm_verify_signature" => abi_evm_verify_signature,
        "abi_evm_get_address_from_pubkey" => abi_evm_get_address_from_pubkey,
        "abi_evm_get_pubkey_from_signature" => abi_evm_get_pubkey_from_signature,
        "abi_is_address_eoa" => abi_is_address_eoa,
        "abi_chain_id" => abi_chain_id,
        "abi_deferred_call_cancel" => abi_deferred_call_cancel,
        "abi_get_deferred_call_quote" => abi_get_deferred_call_quote,
        "abi_deferred_call_exists" => abi_deferred_call_exists,
        "abi_deferred_call_register" => abi_deferred_call_register
    );

    imports
}

/// Call another smart contract
fn abi_call(store_env: FunctionEnvMut<ABIEnv>, arg_offset: i32) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_call,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler, req: CallRequest| {
            let amount = req
                .call_coins
                .ok_or_else(|| WasmV1Error::RuntimeError("No coins provided".into()))?;

            #[cfg(feature = "execution-trace")]
            let amount_ = Decimal::try_from_i128_with_scale(amount.mantissa as i128, amount.scale)
                .unwrap_or_default();

            let interface = handler.exec_env.get_interface();
            let bytecode = interface
                .init_call_wasmv1(&req.target_sc_address, amount)
                .map_err(|err| {
                    WasmV1Error::RuntimeError(format!("Could not init call: {}", err))
                })?;
            let remaining_gas = handler.get_remaining_gas();
            let interface = handler.exec_env.get_interface();
            let module = helper_get_module(interface, bytecode, remaining_gas)?;
            interface.increment_recursion_counter().map_err(|e| {
                WasmV1Error::RuntimeError(format!("Could not increment recursion counter: {}", e))
            })?;
            let response = crate::execution::run_function(
                interface,
                module,
                &req.target_function_name,
                &req.function_arg,
                remaining_gas,
                handler.get_gas_costs().clone(),
                handler.get_condom_limits().clone(),
            )
            .map_err(|err| WasmV1Error::RuntimeError(format!("Could not run function: {}", err)))?;
            interface.decrement_recursion_counter().map_err(|e| {
                WasmV1Error::RuntimeError(format!("Could not decrement recursion counter: {}", e))
            })?;
            handler.set_remaining_gas(response.remaining_gas);
            let interface = handler.exec_env.get_interface();
            interface.finish_call().map_err(|err| {
                WasmV1Error::RuntimeError(format!("Could not finish call: {}", err))
            })?;

            #[cfg(feature = "execution-trace")]
            {
                if let Some(exec_env) = handler.store_env.data_mut().lock().as_mut() {
                    exec_env.trace.push(AbiTrace {
                        name: "abi_call".to_string(),
                        params: vec![
                            into_trace_value!(req.target_sc_address),
                            into_trace_value!(req.target_function_name),
                            into_trace_value!(req.function_arg),
                            into_trace_value!(amount_.to_i64().unwrap()),
                        ],
                        return_value: AbiTraceType::ByteArray(response.ret.clone()),
                        sub_calls: Some(response.trace),
                    });
                }
            }
            Ok(CallResponse { data: response.ret })
        }
    )
}

/// Alternative to `call_module` to execute bytecode in a local context
/// Reuse the protobuf CallRequest message, the call_coins field is just ignored
fn abi_local_call(store_env: FunctionEnvMut<ABIEnv>, arg_offset: i32) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_local_call,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler, req: CallRequest| {
            let bytecode = helper_get_bytecode(handler, req.target_sc_address.clone())?;
            let remaining_gas = handler.get_remaining_gas();
            let interface = handler.exec_env.get_interface();
            let module = helper_get_module(interface, bytecode.clone(), remaining_gas)?;
            interface.increment_recursion_counter().map_err(|e| {
                WasmV1Error::RuntimeError(format!("Could not increment recursion counter: {}", e))
            })?;
            let response = crate::execution::run_function(
                interface,
                module,
                &req.target_function_name,
                &req.function_arg,
                remaining_gas,
                handler.get_gas_costs().clone(),
                handler.get_condom_limits().clone(),
            )
            .map_err(|err| WasmV1Error::RuntimeError(format!("Could not run function: {}", err)))?;
            interface.decrement_recursion_counter().map_err(|e| {
                WasmV1Error::RuntimeError(format!("Could not decrement recursion counter: {}", e))
            })?;
            handler.set_remaining_gas(response.remaining_gas);

            #[cfg(feature = "execution-trace")]
            {
                if let Some(exec_env) = handler.store_env.data_mut().lock().as_mut() {
                    exec_env.trace.push(AbiTrace {
                        name: "abi_local_call".to_string(),
                        params: vec![
                            into_trace_value!(bytecode),
                            into_trace_value!(req.target_function_name),
                            into_trace_value!(req.function_arg),
                        ],
                        return_value: AbiTraceType::ByteArray(response.ret.clone()),
                        sub_calls: Some(response.trace),
                    });
                }
            }

            Ok(CallResponse { data: response.ret })
        }
    )
}

/// Create a new smart contract.
fn abi_create_sc(store_env: FunctionEnvMut<ABIEnv>, arg_offset: i32) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_create_sc,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: CreateScRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let interface = handler.exec_env.get_interface();
            match interface.create_module(&req.bytecode) {
                Ok(sc_address) => {
                    resp_ok!(CreateScResult, { sc_address })
                }
                Err(e) => resp_err!(e),
            }
        }
    )
}

/// gets the current execution slot
fn abi_get_current_slot(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_get_current_slot,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         _req: GetCurrentSlotRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let interface = handler.exec_env.get_interface();
            match interface.get_current_slot() {
                Ok(slot) => resp_ok!(GetCurrentSlotResult, {
                    slot: Some(slot)
                }),
                Err(e) => resp_err!(e),
            }
        }
    )
}

/// performs a hash on a bytearray and returns the native_hash
fn abi_hash_blake3(store_env: FunctionEnvMut<ABIEnv>, arg_offset: i32) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_hash_blake3,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: HashBlake3Request|
         -> Result<AbiResponse, WasmV1Error> {
            let interface = handler.exec_env.get_interface();
            match interface.hash_blake3(&req.data) {
                Ok(hash) => {
                    resp_ok!(HashBlake3Result, { hash: hash.to_vec() })
                }
                Err(e) => resp_err!(e),
            }
        }
    )
}

/// performs a sha256 hash on byte array and returns the hash as byte array
fn abi_hash_sha256(store_env: FunctionEnvMut<ABIEnv>, arg_offset: i32) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_hash_sha256,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: HashSha256Request|
         -> Result<AbiResponse, WasmV1Error> {
            let interface = handler.exec_env.get_interface();
            match interface.hash_sha256(&req.data) {
                Ok(hash) => resp_ok!(HashSha256Result, { hash: hash.to_vec() }),
                Err(e) => resp_err!(e),
            }
        }
    )
}

/// performs a keccak256 hash on byte array and returns the hash as byte array
fn abi_hash_keccak256(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_hash_keccak256,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: Keccak256Request|
         -> Result<AbiResponse, WasmV1Error> {
            let interface = handler.exec_env.get_interface();
            match interface.hash_keccak256(&req.data) {
                Ok(hash) => resp_ok!(Keccak256Result, { hash: hash.to_vec() }),
                Err(e) => resp_err!(e),
            }
        }
    )
}

/// Function designed to abort execution.
fn abi_transfer_coins(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_transfer_coins,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: TransferCoinsRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let Some(amount) = req.amount_to_transfer else {
                return resp_err!("No coins provided");
            };

            #[cfg(feature = "execution-trace")]
            let amount_ = Decimal::try_from_i128_with_scale(amount.mantissa as i128, amount.scale)
                .unwrap_or_default();

            let interface = handler.exec_env.get_interface();

            match interface.transfer_coins_wasmv1(
                req.target_address.clone(),
                amount,
                req.sender_address.clone(),
            ) {
                Ok(_) => {
                    #[cfg(feature = "execution-trace")]
                    {
                        // Build parameters
                        let mut params = vec![
                            ("target_address", req.target_address).into(),
                            ("amount", amount_.to_i64().unwrap_or_default()).into(),
                        ];

                        let sender_address = match req.sender_address {
                            Some(sender_address) => sender_address,
                            None => {
                                let call_stack = interface.get_call_stack();
                                call_stack
                                    .unwrap_or_default()
                                    .last()
                                    .cloned()
                                    .unwrap_or_default()
                            }
                        };
                        params.push(into_trace_value!(sender_address));

                        // let mut guard = handler.store_env.data_mut().lock();
                        handler.exec_env.trace.push(AbiTrace {
                            name: "abi_transfer_coins".to_string(),
                            params,
                            return_value: AbiTraceType::None,
                            sub_calls: None,
                        });
                    }
                    resp_ok!(TransferCoinsResult, {})
                }
                Err(e) => resp_err!(e),
            }
        }
    )
}

fn abi_generate_event(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_generate_event,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: GenerateEventRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let interface = handler.exec_env.get_interface();
            interface.generate_event_wasmv1(req.event).map_err(|err| {
                WasmV1Error::RuntimeError(format!("Failed to generate event: {}", err))
            })?;

            resp_ok!(GenerateEventResult, {})
        }
    )
}

/// Function designed to abort execution.
fn abi_abort(store_env: FunctionEnvMut<ABIEnv>, arg_offset: i32) -> Result<i32, WasmV1Error> {
    crate::handle_abi_raw!(abi_abort, store_env, arg_offset, |_handler,
                                                              req: Vec<u8>|
     -> Result<
        Vec<u8>,
        WasmV1Error,
    > {
        let msg = format!("Guest program abort: {}", String::from_utf8_lossy(&req));

        Err(WasmV1Error::RuntimeError(msg))
    })
}

fn abi_set_ds_value(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_set_ds_value,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: SetDsValueRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let interface = handler.exec_env.get_interface();
            if let Err(e) = interface.set_ds_value_wasmv1(&req.key, &req.value, None) {
                return resp_err!(e);
            }
            resp_ok!(SetDsValueResult, {})
        }
    )
}

fn abi_get_ds_value(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_get_ds_value,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: GetDsValueRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let interface = handler.exec_env.get_interface();
            match interface.get_ds_value_wasmv1(&req.key, req.address) {
                Ok(value) => resp_ok!(GetDsValueResult, { value }),
                Err(e) => resp_err!(e),
            }
        }
    )
}

fn abi_delete_ds_entry(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_delete_ds_entry,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: DeleteDsEntryRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let interface = handler.exec_env.get_interface();
            if let Err(e) = interface.delete_ds_entry_wasmv1(&req.key, req.address) {
                return resp_err!(e);
            }
            resp_ok!(DeleteDsEntryResult, {})
        }
    )
}

fn abi_append_ds_value(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_append_ds_value,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: AppendDsValueRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let interface = handler.exec_env.get_interface();
            if let Err(e) = interface.append_ds_value_wasmv1(&req.key, &req.value, req.address) {
                return resp_err!(e);
            }
            resp_ok!(AppendDsValueResult, {})
        }
    )
}

fn abi_ds_entry_exists(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_ds_entry_exists,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: DsEntryExistsRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let interface = handler.exec_env.get_interface();
            match interface.ds_entry_exists_wasmv1(&req.key, req.address) {
                Ok(has_data) => resp_ok!(DsEntryExistsResult, { has_data }),
                Err(e) => resp_err!(e),
            }
        }
    )
}

fn abi_get_balance(store_env: FunctionEnvMut<ABIEnv>, arg_offset: i32) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_get_balance,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: GetBalanceRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let interface = handler.exec_env.get_interface();
            match interface.get_balance_wasmv1(req.address) {
                Ok(res) => resp_ok!(GetBalanceResult, { balance: Some(res) }),
                Err(e) => resp_err!(e),
            }
        }
    )
}

fn abi_get_bytecode(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_get_bytecode,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: GetBytecodeRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let interface = handler.exec_env.get_interface();
            match interface.get_bytecode_wasmv1(req.address) {
                Ok(bytecode) => resp_ok!(GetBytecodeResult, { bytecode }),
                Err(e) => resp_err!(e),
            }
        }
    )
}

fn abi_set_bytecode(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_set_bytecode,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: SetBytecodeRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let interface = handler.exec_env.get_interface();
            match interface.set_bytecode_wasmv1(&req.bytecode, req.address) {
                Ok(_) => resp_ok!(SetBytecodeResult, {}),
                Err(e) => resp_err!(e),
            }
        }
    )
}

fn abi_get_ds_keys(store_env: FunctionEnvMut<ABIEnv>, arg_offset: i32) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_get_ds_keys,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: GetDsKeysRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let interface = handler.exec_env.get_interface();
            match interface.get_ds_keys_wasmv1(&req.prefix, req.address) {
                Ok(res) => {
                    resp_ok!(GetDsKeysResult, { keys: res.into_iter().collect()})
                }
                Err(e) => resp_err!(e),
            }
        }
    )
}

fn abi_get_op_keys(store_env: FunctionEnvMut<ABIEnv>, arg_offset: i32) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_get_op_keys,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: GetOpKeysRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let interface = handler.exec_env.get_interface();
            match interface.get_op_keys_wasmv1(&req.prefix) {
                Ok(keys) => resp_ok!(GetOpKeysResult, { keys }),
                Err(e) => resp_err!(e),
            }
        }
    )
}

fn abi_op_entry_exists(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_op_entry_exists,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: OpEntryExistsRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let interface = handler.exec_env.get_interface();
            match interface.op_entry_exists(&req.key) {
                Ok(has_key) => resp_ok!(OpEntryExistsResult, { has_key }),
                Err(e) => resp_err!(e),
            }
        }
    )
}

fn abi_get_op_data(store_env: FunctionEnvMut<ABIEnv>, arg_offset: i32) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_get_op_data,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: GetOpDataRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let interface = handler.exec_env.get_interface();
            match interface.get_op_data(&req.key) {
                Ok(value) => resp_ok!(GetOpDataResult, { value }),
                Err(e) => resp_err!(e),
            }
        }
    )
}

fn abi_evm_verify_signature(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_evm_verify_signature,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: EvmVerifySigRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let interface = handler.exec_env.get_interface();
            match interface.evm_signature_verify(&req.message, &req.sig, &req.pub_key) {
                Ok(is_verified) => {
                    resp_ok!(EvmVerifySigResult, { is_verified })
                }
                _ => {
                    resp_err!("EVM signature verification failed")
                }
            }
        }
    )
}

fn abi_evm_get_address_from_pubkey(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_evm_get_address_from_pubkey,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: EvmGetAddressFromPubkeyRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let interface = handler.exec_env.get_interface();
            match interface.evm_get_address_from_pubkey(&req.pub_key) {
                Ok(address) => {
                    resp_ok!(EvmGetAddressFromPubkeyResult, { address })
                }
                Err(e) => resp_err!(e),
            }
        }
    )
}

fn abi_evm_get_pubkey_from_signature(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_evm_get_pubkey_from_signature,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: EvmGetPubkeyFromSignatureRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let interface = handler.exec_env.get_interface();
            match interface.evm_get_pubkey_from_signature(&req.hash, &req.sig) {
                Ok(pub_key) => {
                    resp_ok!(EvmGetPubkeyFromSignatureResult, { pub_key })
                }
                Err(e) => resp_err!(e),
            }
        }
    )
}

fn abi_is_address_eoa(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_is_address_eoa,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: IsAddressEoaRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let interface = handler.exec_env.get_interface();
            match interface.is_address_eoa(&req.address) {
                Ok(is_eoa) => resp_ok!(IsAddressEoaResult, { is_eoa }),
                Err(e) => resp_err!(e),
            }
        }
    )
}

fn abi_get_remaining_gas(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_get_remaining_gas,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         _req: GetRemainingGasRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let remaining_gas = handler.get_remaining_gas();
            resp_ok!(GetRemainingGasResult, { remaining_gas })
        }
    )
}

fn abi_get_owned_addresses(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_get_owned_addresses,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         _req: GetOwnedAddressesRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let interface = handler.exec_env.get_interface();
            match interface.get_owned_addresses() {
                Ok(addresses) => {
                    resp_ok!(GetOwnedAddressesResult, { addresses })
                }
                Err(e) => resp_err!(e),
            }
        }
    )
}

fn abi_get_call_stack(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_get_call_stack,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         _req: GetCallStackRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let interface = handler.exec_env.get_interface();
            match interface.get_call_stack() {
                Ok(calls) => {
                    resp_ok!(GetCallStackResult, { calls })
                }
                Err(e) => resp_err!(e),
            }
        }
    )
}

fn abi_address_from_public_key(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_address_from_public_key,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: AddressFromPubKeyRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let interface = handler.exec_env.get_interface();
            match interface.address_from_public_key(&req.pub_key) {
                Ok(address) => {
                    resp_ok!(AddressFromPubKeyResult, { address })
                }
                Err(e) => resp_err!(e),
            }
        }
    )
}

fn abi_unsafe_random(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_unsafe_random,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: UnsafeRandomRequest|
         -> Result<AbiResponse, WasmV1Error> {
            if req.num_bytes as u64 > handler.get_max_mem_size() {
                return resp_err!("Requested random bytes exceed the maximum memory size");
            }
            let interface = handler.exec_env.get_interface();

            match interface.unsafe_random_wasmv1(req.num_bytes as u64) {
                Ok(random_bytes) => {
                    resp_ok!(UnsafeRandomResult, { random_bytes })
                }
                Err(e) => resp_err!(e),
            }
        }
    )
}

fn abi_get_call_coins(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_get_call_coins,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         _req: GetCallCoinsRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let interface = handler.exec_env.get_interface();
            match interface.get_call_coins_wasmv1() {
                Ok(coins) => {
                    resp_ok!(GetCallCoinsResult, { coins: Some(coins) })
                }
                Err(e) => resp_err!(e),
            }
        }
    )
}

fn abi_get_native_time(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_get_native_time,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         _req: GetNativeTimeRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let interface = handler.exec_env.get_interface();
            match interface.get_time() {
                Err(e) => resp_err!(e),
                Ok(time) => {
                    resp_ok!(GetNativeTimeResult, { time: Some(NativeTime { milliseconds: time }) })
                }
            }
        }
    )
}

fn abi_deferred_call_cancel(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_deferred_call_cancel,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: DeferredCallCancelRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let Some(call_id) = req.call_id else {
                return resp_err!("Call ID is required");
            };

            let interface: &dyn Interface = handler.exec_env.get_interface();
            match interface.deferred_call_cancel(&call_id) {
                Ok(_) => {
                    #[cfg(feature = "execution-trace")]
                    {
                        let params = vec![into_trace_value!(call_id)];
                        if let Some(exec_env) = handler.store_env.data_mut().lock().as_mut() {
                            exec_env.trace.push(AbiTrace {
                                name: "abi_deferred_call_cancel".to_string(),
                                params,
                                return_value: AbiTraceType::None,
                                sub_calls: None,
                            });
                        }
                    }

                    resp_ok!(DeferredCallCancelResult, {})
                }
                Err(e) => resp_err!(e),
            }
        }
    )
}

fn abi_deferred_call_register(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_deferred_call_register,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: DeferredCallRegisterRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let Some(target_slot) = req.target_slot else {
                return resp_err!("Target slot is required");
            };

            let Ok(target_thread): Result<u8, _> = target_slot.thread.try_into() else {
                return resp_err!("Invalid target thread");
            };

            let interface = handler.exec_env.get_interface();
            match interface.deferred_call_register(
                &req.target_address,
                &req.target_function,
                (target_slot.period, target_thread),
                req.max_gas,
                &req.params,
                req.coins,
            ) {
                Ok(call_id) => {
                    #[cfg(feature = "execution-trace")]
                    {
                        let params = vec![
                            into_trace_value!(req.target_address),
                            into_trace_value!(req.target_function),
                            into_trace_value!(target_slot.period),
                            into_trace_value!(target_slot.thread),
                            into_trace_value!(req.max_gas),
                            into_trace_value!(req.coins),
                            into_trace_value!(req.params),
                        ];
                        if let Some(exec_env) = handler.store_env.data_mut().lock().as_mut() {
                            exec_env.trace.push(AbiTrace {
                                name: "abi_deferred_call_register".to_string(),
                                params,
                                return_value: AbiTraceType::String(call_id.clone()),
                                sub_calls: None,
                            });
                        }
                    }

                    resp_ok!(DeferredCallRegisterResult, {call_id: Some(call_id)})
                }
                Err(e) => resp_err!(e),
            }
        }
    )
}

fn abi_get_deferred_call_quote(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_get_deferred_call_quote,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: DeferredCallQuoteRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let interface = handler.exec_env.get_interface();

            let Some(target_slot) = req.target_slot else {
                return resp_err!("Target slot is required");
            };

            match interface.get_deferred_call_quote(
                (target_slot.period, target_slot.thread as u8),
                req.max_gas,
                req.params_size,
            ) {
                Ok((available, mut price)) => {
                    if !available {
                        price = 0;
                    }
                    resp_ok!(DeferredCallQuoteResult, { available, cost: price })
                }
                Err(e) => resp_err!(e),
            }
        }
    )
}

fn abi_deferred_call_exists(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_deferred_call_exists,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: DeferredCallExistsRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let Some(call_id) = req.call_id else {
                return resp_err!("Call ID is required");
            };

            let interface = handler.exec_env.get_interface();
            match interface.deferred_call_exists(&call_id) {
                Ok(exists) => {
                    resp_ok!(DeferredCallExistsResult, { call_exists: exists })
                }
                Err(e) => resp_err!(e),
            }
        }
    )
}

fn abi_send_async_message(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_send_async_message,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: SendAsyncMessageRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let Some(start) = req.validity_start else {
                return resp_err!("Validity start slot is required");
            };
            let Some(end) = req.validity_end else {
                return resp_err!("Validity end slot is required");
            };
            let Ok(start_thread): Result<u8, _> = start.thread.try_into() else {
                return resp_err!("Invalid start thread");
            };
            let Ok(end_thread): Result<u8, _> = end.thread.try_into() else {
                return resp_err!("Invalid end thread");
            };

            let filter: Option<(&str, Option<&[u8]>)> = req
                .filter
                .as_ref()
                .map(|f| (f.target_address.as_str(), f.target_key.as_deref()));

            let interface = handler.exec_env.get_interface();
            match interface.send_message(
                &req.target_address,
                &req.target_handler,
                (start.period, start_thread),
                (end.period, end_thread),
                req.execution_gas,
                req.raw_fee,
                req.raw_coins,
                &req.data,
                filter,
            ) {
                Ok(_) => {
                    #[cfg(feature = "execution-trace")]
                    {
                        let filter_key = filter.unwrap_or_default().1.unwrap_or_default().to_vec();
                        let filter = filter.unwrap_or_default().0.to_string();
                        let params = vec![
                            into_trace_value!(req.target_address),
                            into_trace_value!(req.target_handler),
                            into_trace_value!(start.period),
                            into_trace_value!(start.thread),
                            into_trace_value!(end.period),
                            into_trace_value!(end.thread),
                            into_trace_value!(req.execution_gas),
                            into_trace_value!(req.raw_fee),
                            into_trace_value!(req.raw_coins),
                            into_trace_value!(req.data),
                            // filter address
                            into_trace_value!(filter),
                            // filter key
                            into_trace_value!(filter_key),
                        ];
                        if let Some(exec_env) = handler.store_env.data_mut().lock().as_mut() {
                            exec_env.trace.push(AbiTrace {
                                name: "abi_send_async_message".to_string(),
                                params,
                                return_value: AbiTraceType::None,
                                sub_calls: None,
                            });
                        }
                    }

                    resp_ok!(SendAsyncMessageResult, {})
                }
                Err(e) => resp_err!(e),
            }
        }
    )
}

fn abi_get_origin_operation_id(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_get_origin_operation_id,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         _req: GetOriginOperationIdRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let interface = handler.exec_env.get_interface();
            match interface.get_origin_operation_id() {
                Ok(operation_id) => {
                    resp_ok!(GetOriginOperationIdResult, { operation_id })
                }
                Err(e) => resp_err!(e),
            }
        }
    )
}

fn abi_local_execution(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_local_execution,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: LocalExecutionRequest| {
            let remaining_gas = handler.get_remaining_gas();
            let module = helper_get_tmp_module(handler, req.bytecode.clone(), remaining_gas)?;

            let interface = handler.exec_env.get_interface();
            interface.increment_recursion_counter().map_err(|e| {
                WasmV1Error::RuntimeError(format!("Could not increment recursion counter: {}", e))
            })?;
            match crate::execution::run_function(
                interface,
                module,
                &req.target_function_name,
                &req.function_arg,
                remaining_gas,
                handler.get_gas_costs().clone(),
                handler.get_condom_limits().clone(),
            ) {
                Ok(response) => {
                    interface.decrement_recursion_counter().map_err(|e| {
                        WasmV1Error::RuntimeError(format!(
                            "Could not decrement recursion counter: {}",
                            e
                        ))
                    })?;
                    handler.set_remaining_gas(response.remaining_gas);

                    #[cfg(feature = "execution-trace")]
                    {
                        if let Some(exec_env) = handler.store_env.data_mut().lock().as_mut() {
                            exec_env.trace.push(AbiTrace {
                                name: "abi_local_execution".to_string(),
                                params: vec![
                                    into_trace_value!(req.bytecode),
                                    into_trace_value!(req.target_function_name),
                                    into_trace_value!(req.function_arg),
                                ],
                                return_value: AbiTraceType::ByteArray(response.ret.clone()),
                                sub_calls: Some(response.trace),
                            });
                        }
                    }

                    resp_ok!(LocalExecutionResponse, { data: response.ret })
                }
                Err(e) => resp_err!(e),
            }
        }
    )
}

fn abi_caller_has_write_access(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_caller_has_write_access,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         _req: CallerHasWriteAccessRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let interface = handler.exec_env.get_interface();
            match interface.caller_has_write_access() {
                Ok(has_write_access) => {
                    resp_ok!(CallerHasWriteAccessResult, { has_write_access })
                }
                Err(e) => resp_err!(e),
            }
        }
    )
}

/// Check the exports of a compiled module to see if it contains the given
/// function
fn abi_function_exists(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_function_exists,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: FunctionExistsRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let Ok(bytecode) = helper_get_bytecode(handler, req.target_sc_address) else {
                return resp_err!("No SC found at the given address");
            };

            let remaining_gas = if cfg!(feature = "gas_calibration") {
                u64::MAX
            } else {
                handler.get_remaining_gas()
            };

            // FIXME set updated value to store_env
            let interface = handler.exec_env.get_interface();
            let Ok(module) = helper_get_module(interface, bytecode, remaining_gas) else {
                return resp_ok!(FunctionExistsResult, {
                    exists: false
                });
            };

            resp_ok!(FunctionExistsResult, {
                exists: module.function_exists(&req.function_name) })
        }
    )
}

fn helper_get_bytecode(
    handler: &mut super::handler::ABIHandler,
    address: String,
) -> Result<Vec<u8>, WasmV1Error> {
    let interface = handler.exec_env.get_interface();
    let bytecode = interface.raw_get_bytecode_for(&address).map_err(|err| {
        WasmV1Error::RuntimeError(format!(
            "Could not get bytecode for address: {}: {}",
            address, err
        ))
    })?;
    Ok(bytecode)
}

fn helper_get_module(
    // handler: &mut super::handler::ABIHandler,
    interface: &dyn Interface,
    bytecode: Vec<u8>,
    remaining_gas: u64,
) -> Result<crate::RuntimeModule, WasmV1Error> {
    // let interface = handler.exec_env.get_interface();
    interface
        .get_module(&bytecode, remaining_gas)
        .map_err(|err| WasmV1Error::RuntimeError(format!("Could not get module: {}", err)))
}

fn helper_get_tmp_module(
    handler: &mut super::handler::ABIHandler,
    bytecode: Vec<u8>,
    remaining_gas: u64,
) -> Result<crate::RuntimeModule, WasmV1Error> {
    let interface = handler.exec_env.get_interface();
    interface
        .get_tmp_module(&bytecode, remaining_gas)
        .map_err(|err| WasmV1Error::RuntimeError(format!("Could not get module: {}", err)))
}

fn abi_check_native_amount(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_check_native_amount,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: CheckNativeAmountRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let Some(amount) = req.to_check else {
                return resp_err!("No amount to check");
            };

            let interface = handler.exec_env.get_interface();
            match interface.check_native_amount_wasmv1(&amount) {
                Ok(is_valid) => {
                    resp_ok!(CheckNativeAmountResult, { is_valid })
                }
                Err(e) => resp_err!(e),
            }
        }
    )
}

fn abi_add_native_amount(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_add_native_amount,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: AddNativeAmountRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let Some(amount1) = req.amount1 else {
                return resp_err!("No amount1");
            };
            let Some(amount2) = req.amount2 else {
                return resp_err!("No amount2");
            };

            let interface = handler.exec_env.get_interface();
            match interface.add_native_amount_wasmv1(&amount1, &amount2) {
                Ok(res) => {
                    resp_ok!(AddNativeAmountResult, { sum: Some(res)})
                }
                Err(e) => resp_err!(e),
            }
        }
    )
}

fn abi_sub_native_amount(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_sub_native_amount,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: SubNativeAmountRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let Some(left) = req.left else {
                return resp_err!("No left amount");
            };
            let Some(right) = req.right else {
                return resp_err!("No right amount");
            };

            let interface = handler.exec_env.get_interface();
            match interface.sub_native_amount_wasmv1(&left, &right) {
                Ok(res) => {
                    resp_ok!(SubNativeAmountResult, { difference: Some(res)})
                }
                Err(e) => resp_err!(e),
            }
        }
    )
}

fn abi_scalar_mul_native_amount(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_scalar_mul_native_amount,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: ScalarMulNativeAmountRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let Some(amount) = req.amount else {
                return resp_err!("No amount");
            };

            let interface = handler.exec_env.get_interface();
            match interface.scalar_mul_native_amount_wasmv1(&amount, req.coefficient) {
                Ok(res) => {
                    resp_ok!(ScalarMulNativeAmountResult, { product: Some(res) })
                }
                Err(e) => resp_err!(e),
            }
        }
    )
}

fn abi_scalar_div_rem_native_amount(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_scalar_div_rem_native_amount,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: ScalarDivRemNativeAmountRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let Some(dividend) = req.dividend else {
                return resp_err!("No dividend");
            };

            let interface = handler.exec_env.get_interface();
            match interface.scalar_div_rem_native_amount_wasmv1(&dividend, req.divisor) {
                Ok(res) => {
                    resp_ok!(ScalarDivRemNativeAmountResult,
                        { quotient: Some(res.0), remainder: Some(res.1)})
                }
                Err(e) => resp_err!(e),
            }
        }
    )
}

fn abi_div_rem_native_amount(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_div_rem_native_amount,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: DivRemNativeAmountRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let Some(dividend) = req.dividend else {
                return resp_err!("No dividend");
            };
            let Some(divisor) = req.divisor else {
                return resp_err!("No divisor");
            };

            let interface = handler.exec_env.get_interface();
            match interface.div_rem_native_amount_wasmv1(&dividend, &divisor) {
                Ok(res) => {
                    resp_ok!(DivRemNativeAmountResult,
                        { quotient: res.0, remainder: Some(res.1)})
                }
                Err(e) => resp_err!(e),
            }
        }
    )
}

fn abi_native_amount_to_string(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_native_amount_to_string,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: NativeAmountToStringRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let Some(amount) = req.to_convert else {
                return resp_err!("No amount to convert");
            };

            let interface = handler.exec_env.get_interface();
            match interface.native_amount_to_string_wasmv1(&amount) {
                Ok(converted_amount) => {
                    resp_ok!(NativeAmountToStringResult, { converted_amount })
                }
                Err(e) => resp_err!(e),
            }
        }
    )
}

fn abi_native_amount_from_string(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_native_amount_from_string,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: NativeAmountFromStringRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let interface = handler.exec_env.get_interface();
            let Ok(amount) = interface.native_amount_from_str_wasmv1(&req.to_convert) else {
                return resp_err!("Invalid amount");
            };

            resp_ok!(NativeAmountFromStringResult, { converted_amount: Some(amount) })
        }
    )
}

fn abi_base58_check_to_bytes(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_base58_check_to_bytes,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: Base58CheckToBytesRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let interface = handler.exec_env.get_interface();
            match interface.base58_check_to_bytes_wasmv1(&req.base58_check) {
                Ok(bytes) => resp_ok!(Base58CheckToBytesResult, { bytes }),
                Err(e) => resp_err!(e),
            }
        }
    )
}

fn abi_bytes_to_base58_check(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_bytes_to_base58_check,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: BytesToBase58CheckRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let interface = handler.exec_env.get_interface();
            let base58_check = interface.bytes_to_base58_check_wasmv1(&req.bytes);
            resp_ok!(BytesToBase58CheckResult, { base58_check })
        }
    )
}

fn abi_compare_address(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_compare_address,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: CompareAddressRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let interface = handler.exec_env.get_interface();
            match interface.compare_address_wasmv1(&req.left, &req.right) {
                Ok(result) => {
                    resp_ok!(CompareAddressResult, { result : result.into() })
                }
                Err(e) => resp_err!(e),
            }
        }
    )
}

fn abi_compare_native_amount(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_compare_native_amount,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: CompareNativeAmountRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let (Some(left), Some(right)) = (req.left, req.right) else {
                return resp_err!("Either left or right argument is none");
            };
            let interface = handler.exec_env.get_interface();
            match interface.compare_native_amount_wasmv1(&left, &right) {
                Ok(result) => {
                    resp_ok!(CompareNativeAmountResult, { result : result.into() })
                }
                Err(e) => resp_err!(e),
            }
        }
    )
}

fn abi_compare_native_time(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_compare_native_time,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: CompareNativeTimeRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let (Some(left), Some(right)) = (req.left, req.right) else {
                return resp_err!("Either left or right argument is none");
            };
            let interface = handler.exec_env.get_interface();
            match interface.compare_native_time_wasmv1(&left, &right) {
                Ok(result) => {
                    resp_ok!(CompareNativeTimeResult, { result : result.into() })
                }
                Err(e) => resp_err!(e),
            }
        }
    )
}

fn abi_compare_pub_key(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_compare_pub_key,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: ComparePubKeyRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let interface = handler.exec_env.get_interface();
            match interface.compare_pub_key_wasmv1(&req.left, &req.right) {
                Ok(result) => {
                    resp_ok!(ComparePubKeyResult, { result : result.into() })
                }
                Err(e) => resp_err!(e),
            }
        }
    )
}

fn abi_check_address(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_check_address,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: CheckAddressRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let interface = handler.exec_env.get_interface();
            match interface.check_address_wasmv1(&req.to_check) {
                Ok(is_valid) => {
                    resp_ok!(CheckAddressResult, { is_valid })
                }
                Err(e) => resp_err!(e),
            }
        }
    )
}

fn abi_check_pubkey(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_check_pubkey,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: CheckPubKeyRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let interface = handler.exec_env.get_interface();
            match interface.check_pubkey_wasmv1(&req.to_check) {
                Ok(is_valid) => resp_ok!(CheckPubKeyResult, { is_valid }),
                Err(e) => resp_err!(e),
            }
        }
    )
}

fn abi_check_signature(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_check_signature,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: CheckSigRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let interface = handler.exec_env.get_interface();
            match interface.check_signature_wasmv1(&req.to_check) {
                Ok(is_valid) => resp_ok!(CheckSigResult, { is_valid }),
                Err(e) => resp_err!(e),
            }
        }
    )
}

fn abi_get_address_category(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_get_address_category,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: GetAddressCategoryRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let interface = handler.exec_env.get_interface();
            match interface.get_address_category_wasmv1(&req.address) {
                Ok(res) => {
                    resp_ok!(GetAddressCategoryResult, { category: res.into()})
                }
                Err(e) => resp_err!(e),
            }
        }
    )
}

fn abi_get_address_version(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_get_address_version,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: GetAddressVersionRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let interface = handler.exec_env.get_interface();
            match interface.get_address_version_wasmv1(&req.address) {
                Ok(version) => resp_ok!(GetAddressVersionResult, { version }),
                Err(e) => resp_err!(e),
            }
        }
    )
}

fn abi_get_pubkey_version(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_get_pubkey_version,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: GetPubKeyVersionRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let interface = handler.exec_env.get_interface();
            match interface.get_pubkey_version_wasmv1(&req.pub_key) {
                Ok(version) => resp_ok!(GetPubKeyVersionResult, { version }),
                Err(e) => resp_err!(e),
            }
        }
    )
}

fn abi_get_signature_version(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_get_signature_version,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: GetSignatureVersionRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let interface = handler.exec_env.get_interface();
            match interface.get_signature_version_wasmv1(&req.signature) {
                Ok(version) => resp_ok!(GetSignatureVersionResult, { version }),
                Err(e) => resp_err!(e),
            }
        }
    )
}

fn abi_checked_add_native_time(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_checked_add_native_time,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: CheckedAddNativeTimeRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let Some(time1) = req.left else {
                return resp_err!("No time1");
            };
            let Some(time2) = req.right else {
                return resp_err!("No time2");
            };

            let interface = handler.exec_env.get_interface();
            match interface.checked_add_native_time_wasmv1(&time1, &time2) {
                Ok(res) => {
                    resp_ok!(CheckedAddNativeTimeResult, { sum: Some(res)})
                }
                Err(e) => resp_err!(e),
            }
        }
    )
}

fn abi_checked_sub_native_time(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_checked_sub_native_time,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: CheckedSubNativeTimeRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let Some(left) = req.left else {
                return resp_err!("No left time");
            };
            let Some(right) = req.right else {
                return resp_err!("No right time");
            };

            let interface = handler.exec_env.get_interface();
            match interface.checked_sub_native_time_wasmv1(&left, &right) {
                Ok(res) => {
                    resp_ok!(CheckedSubNativeTimeResult, { difference: Some(res)})
                }
                Err(e) => resp_err!(e),
            }
        }
    )
}

fn abi_checked_mul_native_time(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_checked_mul_native_time,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: CheckedScalarMulNativeTimeRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let Some(time) = req.time else {
                return resp_err!("No time");
            };

            let interface = handler.exec_env.get_interface();
            match interface.checked_mul_native_time_wasmv1(&time, req.coefficient) {
                Ok(res) => {
                    resp_ok!(CheckedScalarMulNativeTimeResult, { product: Some(res)})
                }
                Err(e) => resp_err!(e),
            }
        }
    )
}

fn abi_checked_scalar_div_native_time(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_checked_scalar_div_native_time,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: CheckedScalarDivRemNativeTimeRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let Some(dividend) = req.dividend else {
                return resp_err!("No dividend");
            };

            let interface = handler.exec_env.get_interface();
            match interface.checked_scalar_div_native_time_wasmv1(&dividend, req.divisor) {
                Ok(res) => {
                    resp_ok!(CheckedScalarDivRemNativeTimeResult,
                            { quotient: Some(res.0), remainder: Some(res.1)})
                }
                Err(e) => resp_err!(e),
            }
        }
    )
}

fn abi_checked_div_native_time(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_checked_div_native_time,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: CheckedDivRemNativeTimeRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let Some(dividend) = req.dividend else {
                return resp_err!("No dividend");
            };
            let Some(divisor) = req.divisor else {
                return resp_err!("No divisor");
            };

            let interface = handler.exec_env.get_interface();
            match interface.checked_div_native_time_wasmv1(&dividend, &divisor) {
                Ok(res) => {
                    resp_ok!(CheckedDivRemNativeTimeResult,
                            { quotient: res.0, remainder: Some(res.1)})
                }
                Err(e) => resp_err!(e),
            }
        }
    )
}

pub fn abi_verify_signature(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_verify_signature,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         req: VerifySigRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let interface = handler.exec_env.get_interface();
            match interface.signature_verify(&req.message, &req.sig, &req.pub_key) {
                Ok(is_verified) => {
                    resp_ok!(VerifySigResult, { is_verified })
                }
                Err(e) => resp_err!(e),
            }
        }
    )
}

pub fn abi_chain_id(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    crate::handle_abi!(
        abi_chain_id,
        store_env,
        arg_offset,
        |handler: &mut crate::wasmv1_execution::abi::handler::ABIHandler,
         _req: ChainIdRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let interface = handler.exec_env.get_interface();
            match interface.chain_id() {
                Ok(chain_id) => {
                    resp_ok!(ChainIdResult, { id: chain_id })
                }
                Err(e) => resp_err!(e),
            }
        }
    )
}
