use std::vec;

use super::{
    super::{env::ABIEnv, WasmV1Error},
    handler::{handle_abi, handle_abi_raw},
};

use massa_proto_rs::massa::{
    abi::v1::{self as proto, *},
    model::v1::NativeTime,
};

use rand::RngCore;
use wasmer::{
    imports, AsStoreMut, Function, FunctionEnv, FunctionEnvMut, Imports,
};

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
    ($result:tt, { $($field:ident: $value:expr),* $(,)? }) => {
        Ok(AbiResponse {
            resp: Some(abi_response::Resp::Res(RespResult {
                res: Some(resp_result::Res::$result($result {
                    $($field: $value,)*
                }))
            }))
        })
    };
}

/// Register all ABIs to a store
pub fn register_abis(
    store: &mut impl AsStoreMut,
    shared_abi_env: ABIEnv,
) -> Imports {
    let fn_env = FunctionEnv::new(store, shared_abi_env);
    imports! {
        "massa" => {
            "abi_get_remaining_gas" => Function::new_typed_with_env(store, &fn_env, abi_get_remaining_gas),
            "abi_get_owned_addresses" => Function::new_typed_with_env(store, &fn_env, abi_get_owned_addresses),
            "abi_get_call_stack" => Function::new_typed_with_env(store, &fn_env, abi_get_call_stack),
            "abi_address_from_public_key" => Function::new_typed_with_env(store, &fn_env, abi_address_from_public_key),
            "abi_unsafe_random" => Function::new_typed_with_env(store, &fn_env, abi_unsafe_random),
            "abi_get_call_coins" => Function::new_typed_with_env(store, &fn_env, abi_get_call_coins),
            "abi_get_native_time" => Function::new_typed_with_env(store, &fn_env, abi_get_native_time),
            "abi_send_async_message" => Function::new_typed_with_env(store, &fn_env, abi_send_async_message),
            "abi_get_origin_operation_id" => Function::new_typed_with_env(store, &fn_env, abi_get_origin_operation_id),
            "abi_local_execution" => Function::new_typed_with_env(store, &fn_env, abi_local_execution),
            "abi_caller_has_write_access" => Function::new_typed_with_env(store, &fn_env, abi_caller_has_write_access),
            "abi_verify_evm_signature" => Function::new_typed_with_env(store, &fn_env, abi_verify_evm_signature),
            "abi_set_data" => Function::new_typed_with_env(store, &fn_env, abi_set_data),
            "abi_get_data" => Function::new_typed_with_env(store, &fn_env, abi_get_data),
            "abi_delete_data" => Function::new_typed_with_env(store, &fn_env, abi_delete_data),
            "abi_append_data" => Function::new_typed_with_env(store, &fn_env, abi_append_data),
            "abi_has_data" => Function::new_typed_with_env(store, &fn_env, abi_has_data),
            "abi_abort" => Function::new_typed_with_env(store, &fn_env, abi_abort),
            "abi_call" => Function::new_typed_with_env(store, &fn_env, abi_call),
            "abi_local_call" =>
                Function::new_typed_with_env(store, &fn_env, abi_local_call),
            "abi_create_sc" =>
                Function::new_typed_with_env(store, &fn_env, abi_create_sc),
            "abi_transfer_coins" =>
                Function::new_typed_with_env(store, &fn_env, abi_transfer_coins),
            "abi_generate_event" =>
                Function::new_typed_with_env(store, &fn_env, abi_generate_event),
            "abi_function_exists" =>
                Function::new_typed_with_env(store, &fn_env, abi_function_exists),
            "abi_check_native_amount" => Function::new_typed_with_env(
                store,
                &fn_env,
                abi_check_native_amount,
            ),
            "abi_add_native_amounts" =>
                Function::new_typed_with_env(store, &fn_env, abi_add_native_amounts),

            "abi_sub_native_amounts" =>
                Function::new_typed_with_env(store, &fn_env, abi_sub_native_amounts),

            "abi_mul_native_amount" =>
                Function::new_typed_with_env(store, &fn_env, abi_mul_native_amount),

            "abi_div_rem_native_amount" => Function::new_typed_with_env(
                store,
                &fn_env,
                abi_div_rem_native_amount,
            ),
            "abi_div_rem_native_amounts" => Function::new_typed_with_env(
                store,
                &fn_env,
                abi_div_rem_native_amounts,
            ),
            "abi_native_amount_to_string" => Function::new_typed_with_env(
                store,
                &fn_env,
                abi_native_amount_to_string,
            ),
            "abi_native_amount_from_string" => Function::new_typed_with_env(
                store,
                &fn_env,
                abi_native_amount_from_string,
            ),
            "abi_get_current_slot" => Function::new_typed_with_env(
                store,
                &fn_env,
                abi_get_current_slot,
            ),
            "abi_blake3_hash" => Function::new_typed_with_env(
                store,
                &fn_env,
                abi_blake3_hash,
            ),
            "abi_hash_sha256" => Function::new_typed_with_env(
                store,
                &fn_env,
                abi_hash_sha256,
            ),
            "abi_hash_keccak256" => Function::new_typed_with_env(
                store,
                &fn_env,
                abi_hash_keccak256,
            ),
            "abi_get_balance" => Function::new_typed_with_env(
                store,
                &fn_env,
                abi_get_balance,
            ),
            "abi_get_bytecode" => Function::new_typed_with_env(
                store,
                &fn_env,
                abi_get_bytecode,
            ),
            "abi_set_bytecode" => Function::new_typed_with_env(
                store,
                &fn_env,
                abi_set_bytecode,
            ),
            "abi_get_op_keys" => Function::new_typed_with_env(
                store,
                &fn_env,
                abi_get_op_keys,
            ),
            "abi_get_keys" => Function::new_typed_with_env(
                store,
                &fn_env,
                abi_get_keys,
            ),
            "abi_has_op_key" => Function::new_typed_with_env(
                store,
                &fn_env,
                abi_has_op_key,
            ),
            "abi_get_op_data" => Function::new_typed_with_env(
                store,
                &fn_env,
                abi_get_op_data,
            ),
            "abi_check_address" => Function::new_typed_with_env(
                store,
                &fn_env,
                abi_check_address,
            ),
            "abi_check_pubkey" => Function::new_typed_with_env(
                store,
                &fn_env,
                abi_check_pubkey,
            ),
            "abi_check_signature" => Function::new_typed_with_env(
                store,
                &fn_env,
                abi_check_signature,
            ),
            "abi_get_address_category" => Function::new_typed_with_env(
                store,
                &fn_env,
                abi_get_address_category,
            ),
            "abi_get_address_version" => Function::new_typed_with_env(
                store,
                &fn_env,
                abi_get_address_version,
            ),
            "abi_get_pubkey_version" => Function::new_typed_with_env(
                store,
                &fn_env,
                abi_get_pubkey_version,
            ),
            "abi_get_signature_version" => Function::new_typed_with_env(
                store,
                &fn_env,
                abi_get_signature_version,
            ),
            "abi_checked_add_native_time" => Function::new_typed_with_env(
                store,
                &fn_env,
                abi_checked_add_native_time,
            ),
            "abi_checked_sub_native_time" => Function::new_typed_with_env(
                store,
                &fn_env,
                abi_checked_sub_native_time,
            ),
            "abi_checked_mul_native_time" => Function::new_typed_with_env(
                store,
                &fn_env,
                abi_checked_mul_native_time,
            ),
            "abi_checked_scalar_div_native_time" => Function::new_typed_with_env(
                store,
                &fn_env,
                abi_checked_scalar_div_native_time,
            ),
            "abi_checked_div_native_time" => Function::new_typed_with_env(
                store,
                &fn_env,
                abi_checked_div_native_time,
            ),
            "abi_base58_check_to_bytes" => Function::new_typed_with_env(
                store,
                &fn_env,
                abi_base58_check_to_bytes,
            ),
            "abi_bytes_to_base58_check" => Function::new_typed_with_env(
                store,
                &fn_env,
                abi_bytes_to_base58_check,
            ),
            "abi_compare_address" => Function::new_typed_with_env(
                store,
                &fn_env,
                abi_compare_address,
            ),
            "abi_compare_native_amount" => Function::new_typed_with_env(
                store,
                &fn_env,
                abi_compare_native_amount,
            ),
            "abi_compare_native_time" => Function::new_typed_with_env(
                store,
                &fn_env,abi_compare_native_time,),
            "abi_compare_pub_key" => Function::new_typed_with_env(
                store,
                &fn_env,
                abi_compare_pub_key,
            ),
            "abi_date_now" => Function::new_typed_with_env(
                store,
                &fn_env,
                abi_date_now,
            ),
            "abi_process_exit" => Function::new_typed_with_env(
                store,
                &fn_env,
                abi_process_exit,
            ),
            "abi_seed" => Function::new_typed_with_env(
                store,
                &fn_env,
                abi_seed,
            ),
            "abi_verify_signature" => Function::new_typed_with_env(
                store,
                &fn_env,
                abi_verify_signature,
            ),
        },
    }
}

/// Call another smart contract
pub(crate) fn abi_call(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "call",
        store_env,
        arg_offset,
        |handler, req: CallRequest| {
            let amount = req.call_coins.ok_or_else(|| {
                WasmV1Error::RuntimeError("No coins provided".into())
            })?;

            let bytecode = handler
                .interface
                .init_call_wasmv1(&req.target_sc_address, amount)
                .map_err(|err| {
                    WasmV1Error::RuntimeError(format!(
                        "Could not init call: {}",
                        err
                    ))
                })?;

            let remaining_gas = handler.get_remaining_gas();
            let module = helper_get_module(handler, bytecode, remaining_gas)?;
            let response = crate::execution::run_function(
                handler.interface,
                module,
                &req.target_function_name,
                &req.function_arg,
                remaining_gas,
                handler.get_gas_costs().clone(),
            )
            .map_err(|err| {
                WasmV1Error::RuntimeError(format!(
                    "Could not run function: {}",
                    err
                ))
            })?;
            handler.set_remaining_gas(response.remaining_gas);
            handler.interface.finish_call().map_err(|err| {
                WasmV1Error::RuntimeError(format!(
                    "Could not finish call: {}",
                    err
                ))
            })?;
            Ok(CallResponse { data: response.ret })
        },
    )
}

/// Alternative to `call_module` to execute bytecode in a local context
pub(crate) fn abi_local_call(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "local_call",
        store_env,
        arg_offset,
        |handler, req: CallRequest| {
            let bytecode = helper_get_bytecode(handler, req.target_sc_address)?;
            let remaining_gas = handler.get_remaining_gas();
            let module = helper_get_module(handler, bytecode, remaining_gas)?;

            let response = crate::execution::run_function(
                handler.interface,
                module,
                &req.target_function_name,
                &req.function_arg,
                remaining_gas,
                handler.get_gas_costs().clone(),
            )
            .map_err(|err| {
                WasmV1Error::RuntimeError(format!(
                    "Could not run function: {}",
                    err
                ))
            })?;
            handler.set_remaining_gas(response.remaining_gas);

            Ok(CallResponse { data: response.ret })
        },
    )
}

/// Create a new smart contract.
pub(crate) fn abi_create_sc(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "create_sc",
        store_env,
        arg_offset,
        |handler, req: CreateScRequest| -> Result<AbiResponse, WasmV1Error> {
            let addr = handler.interface.create_module(&req.bytecode);

            match addr {
                Ok(addr) => {
                    resp_ok!(CreateScResult, { sc_address: addr })
                }
                Err(err) => resp_err!(err),
            }
        },
    )
}

/// gets the current execution slot
pub(crate) fn abi_get_current_slot(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "get_current_slot",
        store_env,
        arg_offset,
        |handler,
         _req: GetCurrentSlotRequest|
         -> Result<AbiResponse, WasmV1Error> {
            // Do not remove this. It could be used for gas_calibration in
            // future. if cfg!(feature = "gas_calibration") {
            //     let fname = format!("massa.{}:0", function_name!());
            //     param_size_update(&env, &mut ctx, &fname, to_address.len(),
            // true); }

            match handler.interface.get_current_slot() {
                Ok(slot) => resp_ok!(GetCurrentSlotResult, {
                    slot: Some(slot)
                }),
                Err(err) => resp_err!(err),
            }
        },
    )
}

/// performs a hash on a bytearray and returns the native_hash
pub(crate) fn abi_blake3_hash(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "native_hash",
        store_env,
        arg_offset,
        |handler, req: Blake3HashRequest| -> Result<AbiResponse, WasmV1Error> {
            // Do not remove this. It could be used for gas_calibration in
            // future. if cfg!(feature = "gas_calibration") {
            //     let fname = format!("massa.{}:0", function_name!());
            //     param_size_update(&env, &mut ctx, &fname, to_address.len(),
            // true); }

            match handler.interface.blake3_hash(&req.data) {
                Ok(hash) => {
                    resp_ok!(Blake3HashResult, { hash: hash.to_vec() })
                }
                Err(err) => resp_err!(err),
            }
        },
    )
}

/// performs a sha256 hash on byte array and returns the hash as byte array
pub(crate) fn abi_hash_sha256(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "hash_sha256",
        store_env,
        arg_offset,
        |handler, req: HashSha256Request| -> Result<AbiResponse, WasmV1Error> {
            // Do not remove this. It could be used for gas_calibration in
            // future. if cfg!(feature = "gas_calibration") {
            //     let fname = format!("massa.{}:0", function_name!());
            //     param_size_update(&env, &mut ctx, &fname, to_address.len(),
            // true); }

            match handler.interface.hash_sha256(&req.data) {
                Ok(hash) => resp_ok!(HashSha256Result, { hash: hash.to_vec() }),
                Err(err) => resp_err!(err),
            }
        },
    )
}

/// performs a keccak256 hash on byte array and returns the hash as byte array
pub(crate) fn abi_hash_keccak256(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "hash_keccak256",
        store_env,
        arg_offset,
        |handler, req: Keccak256Request| -> Result<AbiResponse, WasmV1Error> {
            // Do not remove this. It could be used for gas_calibration in
            // future. if cfg!(feature = "gas_calibration") {
            //     let fname = format!("massa.{}:0", function_name!());
            //     param_size_update(&env, &mut ctx, &fname, to_address.len(),
            // true); }

            match handler.interface.hash_keccak256(&req.data) {
                Ok(hash) => resp_ok!(Keccak256Result, { hash: hash.to_vec() }),
                Err(err) => resp_err!(err),
            }
        },
    )
}

/// Function designed to abort execution.
pub fn abi_transfer_coins(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "transfer_coins",
        store_env,
        arg_offset,
        |handler,
         req: TransferCoinsRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let Some(amount) = req.amount_to_transfer else {
                return resp_err!("No coins provided");
            };

            // Do not remove this. It could be used for gas_calibration in
            // future. if cfg!(feature = "gas_calibration") {
            //     let fname = format!("massa.{}:0", function_name!());
            //     param_size_update(&env, &mut ctx, &fname, to_address.len(),
            // true); }

            let transfer_coins = handler.interface.transfer_coins_wasmv1(
                req.target_address,
                amount,
                req.sender_address,
            );
            match transfer_coins {
                Ok(_) => resp_ok!(TransferCoinsResult, {}),
                Err(err) => {
                    resp_err!(format!("Transfer coins failed: {}", err))
                }
            }
        },
    )
}

pub fn abi_generate_event(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "generate_event",
        store_env,
        arg_offset,
        |_handler,
         req: GenerateEventRequest|
         -> Result<AbiResponse, WasmV1Error> {
            _handler
                .interface
                .generate_event_wasmv1(req.event)
                .map_err(|err| {
                    WasmV1Error::RuntimeError(format!(
                        "Failed to generate event: {}",
                        err
                    ))
                })?;

            resp_ok!(GenerateEventResult, {})
        },
    )
}

/// Function designed to abort execution.
fn abi_abort(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi_raw(
        "abi_abort",
        store_env,
        arg_offset,
        |_handler, req: Vec<u8>| -> Result<Vec<u8>, WasmV1Error> {
            let msg = format!(
                "Guest program abort: {}",
                String::from_utf8_lossy(&req)
            );

            Err(WasmV1Error::RuntimeError(msg))
        },
    )
}

fn abi_set_data(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "abi_set_data",
        store_env,
        arg_offset,
        |handler, req: SetDataRequest| -> Result<AbiResponse, WasmV1Error> {
            if let Err(e) = handler
                .interface
                .raw_set_data_wasmv1(&req.key, &req.value, None)
            {
                return resp_err!(format!("Failed to set data: {}", e));
            }
            resp_ok!(SetDataResult, {})
        },
    )
}

fn abi_get_data(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "abi_get_data",
        store_env,
        arg_offset,
        |handler, req: GetDataRequest| -> Result<AbiResponse, WasmV1Error> {
            let Ok(data) =
                handler
                .interface
                .raw_get_data_wasmv1(&req.key, req.address) else {
                    return resp_err!("Failed to get data");
                };
            resp_ok!(GetDataResult, { value: data })
        },
    )
}

fn abi_delete_data(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "abi_delete_data",
        store_env,
        arg_offset,
        |handler, req: DeleteDataRequest| -> Result<AbiResponse, WasmV1Error> {
            if let Err(e) = handler
                .interface
                .raw_delete_data_wasmv1(&req.key, req.address)
            {
                return resp_err!(format!("Failed to delete data: {}", e));
            }
            resp_ok!(DeleteDataResult, {})
        },
    )
}

fn abi_append_data(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "abi_append_data",
        store_env,
        arg_offset,
        |handler, req: AppendDataRequest| -> Result<AbiResponse, WasmV1Error> {
            if let Err(e) = handler.interface.raw_append_data_wasmv1(
                &req.key,
                &req.value,
                req.address,
            ) {
                return resp_err!(format!("Failed to append data: {}", e));
            }
            resp_ok!(AppendDataResult, {})
        },
    )
}

fn abi_has_data(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "abi_has_data",
        store_env,
        arg_offset,
        |handler, req: HasDataRequest| -> Result<AbiResponse, WasmV1Error> {
            let Ok(res) = handler
                .interface
                .has_data_wasmv1(&req.key, req.address) else {
                return resp_err!("Failed to check if data exists");
            };
            resp_ok!(HasDataResult, { has_data: res })
        },
    )
}

fn abi_get_balance(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "abi_get_balance",
        store_env,
        arg_offset,
        |handler, req: GetBalanceRequest| -> Result<AbiResponse, WasmV1Error> {
            let Ok(res) = handler.interface.get_balance_wasmv1(req.address) else
            {
                return resp_err!("Failed to get balance");
            };
            resp_ok!(GetBalanceResult, { balance: Some(res) })
        },
    )
}

fn abi_get_bytecode(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "abi_get_bytecode",
        store_env,
        arg_offset,
        |handler,
         req: GetBytecodeRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let Ok(res) = handler.interface.raw_get_bytecode_wasmv1(req.address) else
            {
                return resp_err!("Failed to get bytecode");
            };
            resp_ok!(GetBytecodeResult, { bytecode: res })
        },
    )
}

fn abi_set_bytecode(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "abi_set_bytecode",
        store_env,
        arg_offset,
        |handler,
         req: SetBytecodeRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let Ok(_) = handler
                .interface
                .raw_set_bytecode_wasmv1(&req.bytecode, req.address) else
            {
                return resp_err!("Failed to set bytecode");
            };
            resp_ok!(SetBytecodeResult, {})
        },
    )
}

fn abi_get_keys(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "abi_get_keys",
        store_env,
        arg_offset,
        |handler, req: GetKeysRequest| -> Result<AbiResponse, WasmV1Error> {
            let Ok(res) = handler.interface.get_keys_wasmv1(&req.prefix, req.address) else
            {
                return resp_err!("Failed to get keys");
            };
            resp_ok!(GetKeysResult, { keys: res.into_iter().collect()})
        },
    )
}

fn abi_get_op_keys(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "abi_get_op_keys",
        store_env,
        arg_offset,
        |handler, req: GetOpKeysRequest| -> Result<AbiResponse, WasmV1Error> {
            let Ok(res) = handler.interface.get_op_keys_wasmv1(&req.prefix) else
            {
                return resp_err!("Failed to get op keys");
            };
            resp_ok!(GetOpKeysResult, { keys: res})
        },
    )
}

fn abi_has_op_key(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "abi_has_op_key",
        store_env,
        arg_offset,
        |handler, req: HasOpKeyRequest| -> Result<AbiResponse, WasmV1Error> {
            let Ok(res) = handler.interface.has_op_key(&req.key) else
            {
                return resp_err!("Failed to check if key exists");
            };
            resp_ok!(HasOpKeyResult, { has_key: res})
        },
    )
}

fn abi_get_op_data(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "abi_get_op_data",
        store_env,
        arg_offset,
        |handler, req: GetOpDataRequest| -> Result<AbiResponse, WasmV1Error> {
            let Ok(res) = handler.interface.get_op_data(&req.key) else
            {
                return resp_err!("Failed to get op data");
            };
            resp_ok!(GetOpDataResult, { value: res})
        },
    )
}

fn abi_verify_evm_signature(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "abi_verify_evm_signature",
        store_env,
        arg_offset,
        |handler,
         req: VerifyEvmSigRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let Ok(is_verified) = handler.interface.verify_evm_signature(&req.message, &req.sig, &req.pub_key) else
            {
                return resp_err!("EVM signature verification failed");
            };
            resp_ok!(VerifyEvmSigResult, { is_verified: is_verified })
        },
    )
}

fn abi_get_remaining_gas(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "abi_get_remaining_gas",
        store_env,
        arg_offset,
        |handler,
         _req: GetRemainingGasRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let gas = handler.get_remaining_gas();
            resp_ok!(GetRemainingGasResult, { remaining_gas: gas })
        },
    )
}

fn abi_get_owned_addresses(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "abi_get_owned_addresses",
        store_env,
        arg_offset,
        |handler,
         _req: GetOwnedAddressesRequest|
         -> Result<AbiResponse, WasmV1Error> {
            match handler.interface.get_owned_addresses() {
                Err(e) => {
                    resp_err!(format!("Failed to get owned addresses: {}", e))
                }
                Ok(addresses) => {
                    resp_ok!(GetOwnedAddressesResult, { addresses: addresses })
                }
            }
        },
    )
}

fn abi_get_call_stack(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "abi_get_call_stack",
        store_env,
        arg_offset,
        |handler,
         _req: GetCallStackRequest|
         -> Result<AbiResponse, WasmV1Error> {
            match handler.interface.get_call_stack() {
                Err(e) => {
                    resp_err!(format!("Failed to get the call stack: {}", e))
                }
                Ok(call_stack) => {
                    resp_ok!(GetCallStackResult, { calls: call_stack })
                }
            }
        },
    )
}

fn abi_address_from_public_key(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "abi_address_from_public_key",
        store_env,
        arg_offset,
        |handler,
         req: AddressFromPubKeyRequest|
         -> Result<AbiResponse, WasmV1Error> {
            match handler.interface.address_from_public_key(&req.pub_key) {
                Err(e) => resp_err!(format!(
                    "Failed to get the address from the public key: {}",
                    e
                )),
                Ok(address) => {
                    resp_ok!(AddressFromPubKeyResult, { address: address })
                }
            }
        },
    )
}

fn abi_unsafe_random(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "abi_unsafe_random",
        store_env,
        arg_offset,
        |handler,
         req: UnsafeRandomRequest|
         -> Result<AbiResponse, WasmV1Error> {
            if req.num_bytes as u64 > handler.get_max_mem_size() {
                return resp_err!(
                    "Requested random bytes exceed the maximum memory size"
                );
            }
            let mut rng = rand::thread_rng();
            let mut bytes: Vec<u8> = vec![0; req.num_bytes as usize];
            rng.fill_bytes(&mut bytes);
            resp_ok!(UnsafeRandomResult, { random_bytes: bytes })
        },
    )
}

fn abi_get_call_coins(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "abi_get_call_coins",
        store_env,
        arg_offset,
        |handler,
         _req: GetCallCoinsRequest|
         -> Result<AbiResponse, WasmV1Error> {
            match handler.interface.get_call_coins_wasmv1() {
                Err(e) => {
                    resp_err!(format!("Failed to get the call coins: {}", e))
                }
                Ok(coins) => {
                    resp_ok!(GetCallCoinsResult, { coins: Some(coins) })
                }
            }
        },
    )
}

fn abi_get_native_time(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "abi_get_native_time",
        store_env,
        arg_offset,
        |handler,
         _req: GetNativeTimeRequest|
         -> Result<AbiResponse, WasmV1Error> {
            match handler.interface.get_time() {
                Err(e) => {
                    resp_err!(format!("Failed to get the time: {}", e))
                }
                Ok(time) => {
                    resp_ok!(GetNativeTimeResult, { time: Some(NativeTime { milliseconds: time }) })
                }
            }
        },
    )
}

fn abi_send_async_message(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "abi_send_async_message",
        store_env,
        arg_offset,
        |handler,
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

            // can't use map here borrowed data needs to outlive the closure
            let filter: Option<(&str, Option<&[u8]>)> =
                if let Some(filter_) = req.filter.as_ref() {
                    Some((
                        filter_.target_address.as_str(),
                        filter_.target_key.as_ref().map(|k| k.as_slice()),
                    ))
                } else {
                    None
                };

            match handler.interface.send_message(
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
                Err(e) => {
                    resp_err!(format!("Failed to send the message: {}", e))
                }
                Ok(_) => {
                    resp_ok!(SendAsyncMessageResult, {})
                }
            }
        },
    )
}

fn abi_get_origin_operation_id(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "abi_get_origin_operation_id",
        store_env,
        arg_offset,
        |handler,
         _req: GetOriginOperationIdRequest|
         -> Result<AbiResponse, WasmV1Error> {
            match handler.interface.get_origin_operation_id() {
                Err(e) => {
                    resp_err!(format!(
                        "Failed to get the origin operation id: {}",
                        e
                    ))
                }
                Ok(id) => {
                    resp_ok!(GetOriginOperationIdResult, { operation_id: id })
                }
            }
        },
    )
}

fn abi_local_execution(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "abi_local_execution",
        store_env,
        arg_offset,
        |handler, req: LocalExecutionRequest| {
            let remaining_gas = handler.get_remaining_gas();
            let module =
                helper_get_module(handler, req.bytecode, remaining_gas)?;

            let Ok(response) = crate::execution::run_function(
                handler.interface,
                module,
                &req.target_function_name,
                &req.function_arg,
                remaining_gas,
                handler.get_gas_costs().clone(),
            ) else {
                return resp_err!("Failed to execute the function locally");
            };
            handler.set_remaining_gas(response.remaining_gas);

            resp_ok!(LocalExecutionResponse, { data: response.ret })
        },
    )
}

fn abi_caller_has_write_access(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "abi_caller_has_write_access",
        store_env,
        arg_offset,
        |handler,
         _req: CallerHasWriteAccessRequest|
         -> Result<AbiResponse, WasmV1Error> {
            match handler.interface.caller_has_write_access() {
                Err(e) => {
                    resp_err!(format!(
                        "Failed to get the caller's write access: {}",
                        e
                    ))
                }
                Ok(has_write_access) => {
                    resp_ok!(CallerHasWriteAccessResult, { has_write_access: has_write_access })
                }
            }
        },
    )
}

/// Check the exports of a compiled module to see if it contains the given
/// function
fn abi_function_exists(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "abi_function_exists",
        store_env,
        arg_offset,
        |handler,
         req: FunctionExistsRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let Ok(bytecode) =
                helper_get_bytecode(handler, req.target_sc_address) else {
                    return resp_err!("No SC found at the given address");
            };

            let remaining_gas = if cfg!(feature = "gas_calibration") {
                u64::MAX
            } else {
                handler.get_remaining_gas()
            };

            let Ok(module) = helper_get_module(handler, bytecode, remaining_gas) else {
                return resp_ok!(FunctionExistsResult, {
                    exists: false
                });

            };

            resp_ok!(FunctionExistsResult, {
                exists: module.function_exists(&req.function_name) })
        },
    )
}

fn helper_get_bytecode(
    handler: &mut super::handler::ABIHandler,
    address: String,
) -> Result<Vec<u8>, WasmV1Error> {
    let bytecode =
        handler
            .interface
            .raw_get_bytecode_for(&address)
            .map_err(|err| {
                WasmV1Error::RuntimeError(format!(
                    "Could not get bytecode for address: {}: {}",
                    address, err
                ))
            })?;
    Ok(bytecode)
}

fn helper_get_module(
    handler: &mut super::handler::ABIHandler,
    bytecode: Vec<u8>,
    remaining_gas: u64,
) -> Result<crate::RuntimeModule, WasmV1Error> {
    let module = handler
        .interface
        .get_module(&bytecode, remaining_gas)
        .map_err(|err| {
            WasmV1Error::RuntimeError(format!("Could not get module: {}", err))
        })?;
    Ok(module)
}

pub fn abi_check_native_amount(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "check_native_amount",
        store_env,
        arg_offset,
        |handler,
         req: CheckNativeAmountRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let Some(amount) = req.to_check else {
                return resp_err!("No amount to check");
            };

            match handler.interface.check_native_amount_wasmv1(&amount) {
                Ok(res) => {
                    return resp_ok!(CheckNativeAmountResult, { is_valid: res})
                }
                Err(err) => {
                    return resp_err!(err);
                }
            };
        },
    )
}
pub fn abi_add_native_amounts(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "add_native_amounts",
        store_env,
        arg_offset,
        |handler,
         req: AddNativeAmountsRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let Some(amount1) = req.amount1 else {
                return resp_err!("No amount1");
            };
            let Some(amount2) = req.amount2 else {
                return resp_err!("No amount2");
            };

            match handler
                .interface
                .add_native_amounts_wasmv1(&amount1, &amount2)
            {
                Ok(res) => {
                    return resp_ok!(AddNativeAmountsResult, { sum: Some(res)});
                }
                Err(err) => {
                    return resp_err!(err);
                }
            };
        },
    )
}
pub fn abi_sub_native_amounts(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "sub_native_amounts",
        store_env,
        arg_offset,
        |handler,
         req: SubNativeAmountsRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let Some(left) = req.left else {
                return resp_err!("No left amount");
            };
            let Some(right) = req.right else {
                return resp_err!("No right amount");
            };

            match handler.interface.sub_native_amounts_wasmv1(&left, &right) {
                Ok(res) => {
                    return resp_ok!(SubNativeAmountsResult, { difference: Some(res)});
                }
                Err(err) => {
                    return resp_err!(err);
                }
            };
        },
    )
}
pub fn abi_mul_native_amount(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "mul_native_amount",
        store_env,
        arg_offset,
        |handler,
         req: MulNativeAmountRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let Some(amount) = req.amount else {
                return resp_err!("No amount");
            };

            match handler
                .interface
                .mul_native_amount_wasmv1(&amount, req.coefficient)
            {
                Ok(res) => {
                    return resp_ok!(MulNativeAmountResult, { product: Some(res)});
                }
                Err(err) => {
                    return resp_err!(err);
                }
            };
        },
    )
}
pub fn abi_div_rem_native_amount(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "div_rem_native_amount",
        store_env,
        arg_offset,
        |handler,
         req: ScalarDivRemNativeAmountRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let Some(dividend) = req.dividend else {
                return resp_err!("No dividend");
            };

            match handler
                .interface
                .div_rem_native_amount_wasmv1(&dividend, req.divisor)
            {
                Ok(res) => {
                    return resp_ok!(ScalarDivRemNativeAmountResult,
                            { quotient: Some(res.0), remainder: Some(res.1)});
                }
                Err(err) => {
                    return resp_err!(err);
                }
            };
        },
    )
}
pub fn abi_div_rem_native_amounts(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "div_rem_native_amounts",
        store_env,
        arg_offset,
        |handler,
         req: DivRemNativeAmountsRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let Some(dividend) = req.dividend else {
                return resp_err!("No dividend");
            };
            let Some(divisor) = req.divisor else {
                return resp_err!("No divisor");
            };

            match handler
                .interface
                .div_rem_native_amounts_wasmv1(&dividend, &divisor)
            {
                Ok(res) => {
                    return resp_ok!(DivRemNativeAmountsResult,
                            { quotient: res.0, remainder: Some(res.1)});
                }
                Err(err) => {
                    return resp_err!(err);
                }
            };
        },
    )
}

pub fn abi_native_amount_to_string(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "native_amount_to_string",
        store_env,
        arg_offset,
        |handler,
         req: NativeAmountToStringRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let Some(amount) = req.to_convert else {
                return resp_err!("No amount to convert");
            };

            // match handler.interface.amount_to_string(amount) {
            match handler.interface.native_amount_to_string_wasmv1(&amount) {
                Ok(amount) => {
                    resp_ok!(NativeAmountToStringResult, { converted_amount: amount })
                }
                Err(err) => resp_err!(err),
            }
        },
    )
}

pub fn abi_native_amount_from_string(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "native_amount_from_string",
        store_env,
        arg_offset,
        |handler,
         req: NativeAmountFromStringRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let Ok(amount) =
                handler
                .interface
                .native_amount_from_str_wasmv1(&req.to_convert) else {
                    return resp_err!("Invalid amount");
                };

            resp_ok!(NativeAmountFromStringResult, { converted_amount: Some(amount) })
        },
    )
}

pub fn abi_base58_check_to_bytes(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "base58_check_to_bytes",
        store_env,
        arg_offset,
        |handler,
         req: Base58CheckToBytesRequest|
         -> Result<AbiResponse, WasmV1Error> {
            match handler
                .interface
                .base58_check_to_bytes_wasmv1(&req.base58_check)
            {
                Ok(bytes) => {
                    resp_ok!(Base58CheckToBytesResult, { bytes: bytes })
                }
                Err(err) => {
                    resp_err!(err)
                }
            }
        },
    )
}

pub fn abi_bytes_to_base58_check(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "bytes_to_base58_check",
        store_env,
        arg_offset,
        |handler,
         req: BytesToBase58CheckRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let s = handler.interface.bytes_to_base58_check_wasmv1(&req.bytes);
            resp_ok!(BytesToBase58CheckResult, { base58_check : s })
        },
    )
}

pub fn abi_compare_address(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "abi_compare_address",
        store_env,
        arg_offset,
        |handler,
         req: CompareAddressRequest|
         -> Result<AbiResponse, WasmV1Error> {
            match handler
                .interface
                .compare_address_wasmv1(&req.left, &req.right)
            {
                Ok(result) => {
                    resp_ok!(CompareAddressResult, { result : result.into() })
                }
                Err(err) => {
                    resp_err!(err)
                }
            }
        },
    )
}

pub fn abi_compare_native_amount(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "abi_compare_native_amount",
        store_env,
        arg_offset,
        |handler,
         req: CompareNativeAmountRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let (Some(left), Some(right)) = (req.left, req.right) else {
                return resp_err!("Either left or right argument is none")
            };
            match handler
                .interface
                .compare_native_amount_wasmv1(&left, &right)
            {
                Ok(result) => {
                    resp_ok!(CompareNativeAmountResult, { result : result.into() })
                }
                Err(err) => {
                    resp_err!(err)
                }
            }
        },
    )
}

pub fn abi_compare_native_time(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "abi_compare_native_time",
        store_env,
        arg_offset,
        |handler,
         req: CompareNativeTimeRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let (Some(left), Some(right)) = (req.left, req.right) else {
                return resp_err!("Either left or right argument is none")
            };
            match handler.interface.compare_native_time_wasmv1(&left, &right) {
                Ok(result) => {
                    resp_ok!(CompareNativeTimeResult, { result : result.into() })
                }
                Err(err) => {
                    resp_err!(err)
                }
            }
        },
    )
}

pub fn abi_compare_pub_key(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "abi_compare_pub_key",
        store_env,
        arg_offset,
        |handler,
         req: ComparePubKeyRequest|
         -> Result<AbiResponse, WasmV1Error> {
            match handler
                .interface
                .compare_pub_key_wasmv1(&req.left, &req.right)
            {
                Ok(result) => {
                    resp_ok!(ComparePubKeyResult, { result : result.into() })
                }
                Err(err) => {
                    resp_err!(err)
                }
            }
        },
    )
}

pub fn abi_check_address(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "check_address",
        store_env,
        arg_offset,
        |handler,
         req: CheckAddressRequest|
         -> Result<AbiResponse, WasmV1Error> {
            match handler.interface.check_address_wasmv1(&req.to_check) {
                Ok(res) => {
                    return resp_ok!(CheckAddressResult, { is_valid: res})
                }
                Err(err) => {
                    return resp_err!(err);
                }
            };
        },
    )
}

pub fn abi_check_pubkey(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "check_pubkey",
        store_env,
        arg_offset,
        |handler,
         req: CheckPubKeyRequest|
         -> Result<AbiResponse, WasmV1Error> {
            match handler.interface.check_pubkey_wasmv1(&req.to_check) {
                Ok(res) => return resp_ok!(CheckPubKeyResult, { is_valid: res}),
                Err(err) => {
                    return resp_err!(err);
                }
            };
        },
    )
}

pub fn abi_check_signature(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "check_signature",
        store_env,
        arg_offset,
        |handler, req: CheckSigRequest| -> Result<AbiResponse, WasmV1Error> {
            match handler.interface.check_signature_wasmv1(&req.to_check) {
                Ok(res) => return resp_ok!(CheckSigResult, { is_valid: res}),
                Err(err) => {
                    return resp_err!(err);
                }
            };
        },
    )
}

pub fn abi_get_address_category(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "get_address_category",
        store_env,
        arg_offset,
        |handler,
         req: GetAddressCategoryRequest|
         -> Result<AbiResponse, WasmV1Error> {
            match handler.interface.get_address_category_wasmv1(&req.address) {
                Ok(res) => {
                    return resp_ok!(GetAddressCategoryResult, { category: res.into()})
                }
                Err(err) => {
                    return resp_err!(err);
                }
            };
        },
    )
}

pub fn abi_get_address_version(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "get_address_version",
        store_env,
        arg_offset,
        |handler,
         req: GetAddressVersionRequest|
         -> Result<AbiResponse, WasmV1Error> {
            match handler.interface.get_address_version_wasmv1(&req.address) {
                Ok(res) => {
                    return resp_ok!(GetAddressVersionResult, { version: res})
                }
                Err(err) => {
                    return resp_err!(err);
                }
            };
        },
    )
}

pub fn abi_get_pubkey_version(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "get_pubkey_version",
        store_env,
        arg_offset,
        |handler,
         req: GetPubKeyVersionRequest|
         -> Result<AbiResponse, WasmV1Error> {
            match handler.interface.get_pubkey_version_wasmv1(&req.pub_key) {
                Ok(res) => {
                    return resp_ok!(GetPubKeyVersionResult, { version: res})
                }
                Err(err) => {
                    return resp_err!(err);
                }
            };
        },
    )
}

pub fn abi_get_signature_version(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "get_signature_version",
        store_env,
        arg_offset,
        |handler,
         req: GetSignatureVersionRequest|
         -> Result<AbiResponse, WasmV1Error> {
            match handler
                .interface
                .get_signature_version_wasmv1(&req.signature)
            {
                Ok(res) => {
                    return resp_ok!(GetSignatureVersionResult, { version: res})
                }
                Err(err) => {
                    return resp_err!(err);
                }
            };
        },
    )
}

pub fn abi_checked_add_native_time(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "checked_add_native_time",
        store_env,
        arg_offset,
        |handler,
         req: CheckedAddNativeTimeRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let Some(time1) = req.left else {
                return resp_err!("No time1");
            };
            let Some(time2) = req.right else {
                return resp_err!("No time2");
            };

            match handler
                .interface
                .checked_add_native_time_wasmv1(&time1, &time2)
            {
                Ok(res) => {
                    return resp_ok!(CheckedAddNativeTimeResult, { sum: Some(res)});
                }
                Err(err) => {
                    return resp_err!(err);
                }
            };
        },
    )
}
pub fn abi_checked_sub_native_time(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "checked_sub_native_time",
        store_env,
        arg_offset,
        |handler,
         req: CheckedSubNativeTimeRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let Some(left) = req.left else {
                return resp_err!("No left time");
            };
            let Some(right) = req.right else {
                return resp_err!("No right time");
            };

            match handler
                .interface
                .checked_sub_native_time_wasmv1(&left, &right)
            {
                Ok(res) => {
                    return resp_ok!(CheckedSubNativeTimeResult, { difference: Some(res)});
                }
                Err(err) => {
                    return resp_err!(err);
                }
            };
        },
    )
}
pub fn abi_checked_mul_native_time(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "checked_mul_native_time",
        store_env,
        arg_offset,
        |handler,
         req: CheckedMulNativeTimeRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let Some(time) = req.time else {
                return resp_err!("No time");
            };

            match handler
                .interface
                .checked_mul_native_time_wasmv1(&time, req.coefficient)
            {
                Ok(res) => {
                    return resp_ok!(CheckedMulNativeTimeResult, { product: Some(res)});
                }
                Err(err) => {
                    return resp_err!(err);
                }
            };
        },
    )
}
pub fn abi_checked_scalar_div_native_time(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "checked_scalar_div_native_time",
        store_env,
        arg_offset,
        |handler,
         req: CheckedScalarDivRemNativeTimeRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let Some(dividend) = req.dividend else {
                return resp_err!("No dividend");
            };

            match handler
                .interface
                .checked_scalar_div_native_time_wasmv1(&dividend, req.divisor)
            {
                Ok(res) => {
                    return resp_ok!(CheckedScalarDivRemNativeTimeResult,
                            { quotient: Some(res.0), remainder: Some(res.1)});
                }
                Err(err) => {
                    return resp_err!(err);
                }
            };
        },
    )
}
pub fn abi_checked_div_native_time(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "checked_div_native_time",
        store_env,
        arg_offset,
        |handler,
         req: CheckedDivRemNativeTimeRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let Some(dividend) = req.dividend else {
                return resp_err!("No dividend");
            };
            let Some(divisor) = req.divisor else {
                return resp_err!("No divisor");
            };

            match handler
                .interface
                .checked_div_native_time_wasmv1(&dividend, &divisor)
            {
                Ok(res) => {
                    return resp_ok!(CheckedDivRemNativeTimeResult,
                            { quotient: res.0, remainder: Some(res.1)});
                }
                Err(err) => {
                    return resp_err!(err);
                }
            };
        },
    )
}

pub fn abi_date_now(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "date_now",
        store_env,
        arg_offset,
        |handler,
         _req: DateNowRequest|
         -> Result<AbiResponse, WasmV1Error> {
            match handler.interface.date_now() {
                Ok(date) => {
                    resp_ok!(DateNowResult, { date_now: date })
                }
                Err(err) => {
                    resp_err!(err)
                }
            }
        },
    )
}

pub fn abi_process_exit(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "process_exit",
        store_env,
        arg_offset,
        |_handler,
         req: ProcessExitRequest|
         -> Result<AbiResponse, WasmV1Error> {

            let msg = format!(
                "Guest process exited with code: {}",
                &req.code
            );

            Err(WasmV1Error::RuntimeError(msg))
        },
    )
}

pub fn abi_seed(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "seed",
        store_env,
        arg_offset,
        |handler,
         _req: SeedRequest|
         -> Result<AbiResponse, WasmV1Error> {
            match handler.interface.seed() {
                Ok(seed) => {
                    resp_ok!(SeedResult, { seed: seed })
                }
                Err(err) => {
                    resp_err!(err)
                }
            }
        },
    )
}
pub fn abi_verify_signature(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "verify_signature",
        store_env,
        arg_offset,
        |handler,
         req: VerifySigRequest|
         -> Result<AbiResponse, WasmV1Error> {
            match handler.interface.signature_verify(&req.message, &req.sig, &req.pub_key) {
                Ok(is_verified) => {
                    resp_ok!(VerifySigResult, { is_verified: is_verified })
                },
                Err(err) => {
                    resp_err!(err)
                }
            }
        },
    )
}
