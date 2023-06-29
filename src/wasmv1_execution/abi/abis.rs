use super::{
    super::{env::ABIEnv, WasmV1Error},
    handler::{handle_abi, handle_abi_raw},
};

use massa_proto_rs::massa::{
    abi::v1::{self as proto, *},
    model::v1::*,
};

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

            let amount = handler
                .interface
                .amount_from_mantissa_scale(amount.mantissa, amount.scale)
                .map_err(|err| WasmV1Error::RuntimeError(err.to_string()))?;

            let bytecode = handler
                .interface
                .init_call(&req.target_sc_address, amount)
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
                req.optional_sender_address,
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
            dbg!(&msg);

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
                .raw_get_data_wasmv1(&req.key, req.optional_address) else {
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
                .raw_delete_data_wasmv1(&req.key, req.optional_address)
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
                req.optional_address,
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
                .has_data_wasmv1(&req.key, req.optional_address) else {
                return resp_err!("Failed to check if data exists");
            };
            resp_ok!(HasDataResult, { has_data: res })
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
            let Ok(gas) = handler.get_remaining_gas().try_into() else {
                return resp_err!("Remaining gas is too large to fit into a i64");
            };
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
            // TODO
            resp_ok!(UnsafeRandomResult, {})
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
            match handler.interface.get_call_coins() {
                Err(e) => {
                    resp_err!(format!("Failed to get the call coins: {}", e))
                }
                Ok(coins) => {
                    resp_ok!(GetCallCoinsResult, { coins: Some(NativeAmount { mantissa: coins, scale: 0 }) })
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

fn abi_local_execution(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "local_call",
        store_env,
        arg_offset,
        |handler, req: LocalExecutionRequest| {
            let remaining_gas = handler.get_remaining_gas();
            let module =
                helper_get_module(handler, req.bytecode, remaining_gas)?;

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

            Ok(LocalExecutionResponse { data: response.ret })
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
        |_handler,
         req: CheckNativeAmountRequest|
         -> Result<AbiResponse, WasmV1Error> { todo!() },
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
        |_handler,
         req: AddNativeAmountsRequest|
         -> Result<AbiResponse, WasmV1Error> { todo!() },
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
        |_handler,
         req: SubNativeAmountsRequest|
         -> Result<AbiResponse, WasmV1Error> { todo!() },
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
        |_handler,
         req: MulNativeAmountRequest|
         -> Result<AbiResponse, WasmV1Error> { todo!() },
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
        |_handler,
         req: DivRemNativeAmountRequest|
         -> Result<AbiResponse, WasmV1Error> { todo!() },
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
        |_handler,
         req: DivRemNativeAmountRequest|
         -> Result<AbiResponse, WasmV1Error> { todo!() },
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

            let Ok(amount) =
                handler.interface.amount_from_mantissa_scale(
                    amount.mantissa,
                    amount.scale,
                ) else {
                    return resp_err!("Invalid amount");
                };

            match handler.interface.amount_to_string(amount) {
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
                handler.interface.amount_from_str(&req.to_convert) else {
                    return resp_err!("Invalid amount");
                };
            let Ok((mantissa, scale))=
                handler.interface.amount_to_mantissa_scale(amount) else {
                    return resp_err!("Invalid amount");
                };
            let amount = NativeAmount { mantissa, scale };

            resp_ok!(NativeAmountFromStringResult, { converted_amount: Some(amount) })
        },
    )
}

enum Category {
    Unspecified,
    User,
    SC,
}

fn check_category(cat: Category) -> Result<(), ()> {
    match cat {
        // match know values
        Category::User => Ok(()),
        Category::SC => Ok(()),

        // any invalid value
        _ => Err(()),
    }
}
