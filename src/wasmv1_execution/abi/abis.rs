use super::{
    super::{env::ABIEnv, WasmV1Error},
    handler::{handle_abi, handle_abi_raw},
};
use function_name::named;
use massa_proto_rs::massa::{
    abi::v1::{self as proto, *},
    model::v1::NativeTime,
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
pub fn register_abis(
    store: &mut impl AsStoreMut,
    shared_abi_env: ABIEnv,
) -> Imports {
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

    abis!(
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
        "abi_verify_evm_signature" => abi_verify_evm_signature,
        "abi_verify_signature" => abi_verify_signature
    )
}

/// Call another smart contract
#[named]
fn abi_call(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
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
/// Reuse the protobuf CallRequest message, the call_coins field is just ignored
#[named]
fn abi_local_call(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
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
#[named]
fn abi_create_sc(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
        store_env,
        arg_offset,
        |handler, req: CreateScRequest| -> Result<AbiResponse, WasmV1Error> {
            match handler.interface.create_module(&req.bytecode) {
                Ok(sc_address) => {
                    resp_ok!(CreateScResult, { sc_address })
                }
                Err(e) => resp_err!(e),
            }
        },
    )
}

/// gets the current execution slot
#[named]
fn abi_get_current_slot(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
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
                Err(e) => resp_err!(e),
            }
        },
    )
}

/// performs a hash on a bytearray and returns the native_hash
#[named]
fn abi_hash_blake3(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
        store_env,
        arg_offset,
        |handler, req: HashBlake3Request| -> Result<AbiResponse, WasmV1Error> {
            // Do not remove this. It could be used for gas_calibration in
            // future. if cfg!(feature = "gas_calibration") {
            //     let fname = format!("massa.{}:0", function_name!());
            //     param_size_update(&env, &mut ctx, &fname, to_address.len(),
            // true); }

            match handler.interface.hash_blake3(&req.data) {
                Ok(hash) => {
                    resp_ok!(HashBlake3Result, { hash: hash.to_vec() })
                }
                Err(e) => resp_err!(e),
            }
        },
    )
}

/// performs a sha256 hash on byte array and returns the hash as byte array
#[named]
fn abi_hash_sha256(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
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
                Err(e) => resp_err!(e),
            }
        },
    )
}

/// performs a keccak256 hash on byte array and returns the hash as byte array
#[named]
fn abi_hash_keccak256(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
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
                Err(e) => resp_err!(e),
            }
        },
    )
}

/// Function designed to abort execution.
#[named]
fn abi_transfer_coins(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
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

            match handler.interface.transfer_coins_wasmv1(
                req.target_address,
                amount,
                req.sender_address,
            ) {
                Ok(_) => resp_ok!(TransferCoinsResult, {}),
                Err(e) => resp_err!(e),
            }
        },
    )
}

#[named]
fn abi_generate_event(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
        store_env,
        arg_offset,
        |handler,
         req: GenerateEventRequest|
         -> Result<AbiResponse, WasmV1Error> {
            handler.interface.generate_event_wasmv1(req.event).map_err(
                |err| {
                    WasmV1Error::RuntimeError(format!(
                        "Failed to generate event: {}",
                        err
                    ))
                },
            )?;

            resp_ok!(GenerateEventResult, {})
        },
    )
}

/// Function designed to abort execution.
#[named]
fn abi_abort(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi_raw(
        function_name!(),
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

#[named]
fn abi_set_ds_value(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
        store_env,
        arg_offset,
        |handler, req: SetDsValueRequest| -> Result<AbiResponse, WasmV1Error> {
            if let Err(e) = handler
                .interface
                .set_ds_value_wasmv1(&req.key, &req.value, None)
            {
                return resp_err!(e);
            }
            resp_ok!(SetDsValueResult, {})
        },
    )
}

#[named]
fn abi_get_ds_value(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
        store_env,
        arg_offset,
        |handler, req: GetDsValueRequest| -> Result<AbiResponse, WasmV1Error> {
            match handler.interface.get_ds_value_wasmv1(&req.key, req.address) {
                Ok(value) => resp_ok!(GetDsValueResult, { value }),
                Err(e) => resp_err!(e),
            }
        },
    )
}

#[named]
fn abi_delete_ds_entry(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
        store_env,
        arg_offset,
        |handler,
         req: DeleteDsEntryRequest|
         -> Result<AbiResponse, WasmV1Error> {
            if let Err(e) = handler
                .interface
                .delete_ds_entry_wasmv1(&req.key, req.address)
            {
                return resp_err!(e);
            }
            resp_ok!(DeleteDsEntryResult, {})
        },
    )
}

#[named]
fn abi_append_ds_value(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
        store_env,
        arg_offset,
        |handler,
         req: AppendDsValueRequest|
         -> Result<AbiResponse, WasmV1Error> {
            if let Err(e) = handler.interface.append_ds_value_wasmv1(
                &req.key,
                &req.value,
                req.address,
            ) {
                return resp_err!(e);
            }
            resp_ok!(AppendDsValueResult, {})
        },
    )
}

#[named]
fn abi_ds_entry_exists(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
        store_env,
        arg_offset,
        |handler,
         req: DsEntryExistsRequest|
         -> Result<AbiResponse, WasmV1Error> {
            match handler
                .interface
                .ds_entry_exists_wasmv1(&req.key, req.address)
            {
                Ok(has_data) => resp_ok!(DsEntryExistsResult, { has_data }),
                Err(e) => resp_err!(e),
            }
        },
    )
}

#[named]
fn abi_get_balance(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
        store_env,
        arg_offset,
        |handler, req: GetBalanceRequest| -> Result<AbiResponse, WasmV1Error> {
            match handler.interface.get_balance_wasmv1(req.address) {
                Ok(res) => resp_ok!(GetBalanceResult, { balance: Some(res) }),
                Err(e) => resp_err!(e),
            }
        },
    )
}

#[named]
fn abi_get_bytecode(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
        store_env,
        arg_offset,
        |handler,
         req: GetBytecodeRequest|
         -> Result<AbiResponse, WasmV1Error> {
            match handler.interface.get_bytecode_wasmv1(req.address) {
                Ok(bytecode) => resp_ok!(GetBytecodeResult, { bytecode }),
                Err(e) => resp_err!(e),
            }
        },
    )
}

#[named]
fn abi_set_bytecode(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
        store_env,
        arg_offset,
        |handler,
         req: SetBytecodeRequest|
         -> Result<AbiResponse, WasmV1Error> {
            match handler
                .interface
                .set_bytecode_wasmv1(&req.bytecode, req.address)
            {
                Ok(_) => resp_ok!(SetBytecodeResult, {}),
                Err(e) => resp_err!(e),
            }
        },
    )
}

#[named]
fn abi_get_ds_keys(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
        store_env,
        arg_offset,
        |handler, req: GetDsKeysRequest| -> Result<AbiResponse, WasmV1Error> {
            match handler
                .interface
                .get_ds_keys_wasmv1(&req.prefix, req.address)
            {
                Ok(res) => {
                    resp_ok!(GetDsKeysResult, { keys: res.into_iter().collect()})
                }
                Err(e) => resp_err!(e),
            }
        },
    )
}

#[named]
fn abi_get_op_keys(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
        store_env,
        arg_offset,
        |handler, req: GetOpKeysRequest| -> Result<AbiResponse, WasmV1Error> {
            match handler.interface.get_op_keys_wasmv1(&req.prefix) {
                Ok(keys) => resp_ok!(GetOpKeysResult, { keys }),
                Err(e) => resp_err!(e),
            }
        },
    )
}

#[named]
fn abi_op_entry_exists(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
        store_env,
        arg_offset,
        |handler,
         req: OpEntryExistsRequest|
         -> Result<AbiResponse, WasmV1Error> {
            match handler.interface.op_entry_exists(&req.key) {
                Ok(has_key) => resp_ok!(OpEntryExistsResult, { has_key }),
                Err(e) => resp_err!(e),
            }
        },
    )
}

#[named]
fn abi_get_op_data(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
        store_env,
        arg_offset,
        |handler, req: GetOpDataRequest| -> Result<AbiResponse, WasmV1Error> {
            match handler.interface.get_op_data(&req.key) {
                Ok(value) => resp_ok!(GetOpDataResult, { value }),
                Err(e) => resp_err!(e),
            }
        },
    )
}

#[named]
fn abi_verify_evm_signature(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
        store_env,
        arg_offset,
        |handler,
         req: VerifyEvmSigRequest|
         -> Result<AbiResponse, WasmV1Error> {
            match handler.interface.verify_evm_signature(
                &req.message,
                &req.sig,
                &req.pub_key,
            ) {
                Ok(is_verified) => {
                    resp_ok!(VerifyEvmSigResult, { is_verified })
                }
                _ => {
                    resp_err!("EVM signature verification failed")
                }
            }
        },
    )
}

#[named]
fn abi_get_remaining_gas(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
        store_env,
        arg_offset,
        |handler,
         _req: GetRemainingGasRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let remaining_gas = handler.get_remaining_gas();
            resp_ok!(GetRemainingGasResult, { remaining_gas })
        },
    )
}

#[named]
fn abi_get_owned_addresses(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
        store_env,
        arg_offset,
        |handler,
         _req: GetOwnedAddressesRequest|
         -> Result<AbiResponse, WasmV1Error> {
            match handler.interface.get_owned_addresses() {
                Ok(addresses) => {
                    resp_ok!(GetOwnedAddressesResult, { addresses })
                }
                Err(e) => resp_err!(e),
            }
        },
    )
}

#[named]
fn abi_get_call_stack(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
        store_env,
        arg_offset,
        |handler,
         _req: GetCallStackRequest|
         -> Result<AbiResponse, WasmV1Error> {
            match handler.interface.get_call_stack() {
                Ok(calls) => {
                    resp_ok!(GetCallStackResult, { calls })
                }
                Err(e) => resp_err!(e),
            }
        },
    )
}

#[named]
fn abi_address_from_public_key(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
        store_env,
        arg_offset,
        |handler,
         req: AddressFromPubKeyRequest|
         -> Result<AbiResponse, WasmV1Error> {
            match handler.interface.address_from_public_key(&req.pub_key) {
                Ok(address) => {
                    resp_ok!(AddressFromPubKeyResult, { address })
                }
                Err(e) => resp_err!(e),
            }
        },
    )
}

#[named]
fn abi_unsafe_random(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
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

            match handler.interface.unsafe_random_wasmv1(req.num_bytes as u64) {
                Ok(random_bytes) => {
                    resp_ok!(UnsafeRandomResult, { random_bytes })
                }
                Err(e) => resp_err!(e),
            }
        },
    )
}

#[named]
fn abi_get_call_coins(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
        store_env,
        arg_offset,
        |handler,
         _req: GetCallCoinsRequest|
         -> Result<AbiResponse, WasmV1Error> {
            match handler.interface.get_call_coins_wasmv1() {
                Ok(coins) => {
                    resp_ok!(GetCallCoinsResult, { coins: Some(coins) })
                }
                Err(e) => resp_err!(e),
            }
        },
    )
}

#[named]
fn abi_get_native_time(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
        store_env,
        arg_offset,
        |handler,
         _req: GetNativeTimeRequest|
         -> Result<AbiResponse, WasmV1Error> {
            match handler.interface.get_time() {
                Err(e) => resp_err!(e),
                Ok(time) => {
                    resp_ok!(GetNativeTimeResult, { time: Some(NativeTime { milliseconds: time }) })
                }
            }
        },
    )
}

#[named]
fn abi_send_async_message(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
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

            let filter: Option<(&str, Option<&[u8]>)> = req
                .filter
                .as_ref()
                .map(|f| (f.target_address.as_str(), f.target_key.as_deref()));

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
                Ok(_) => resp_ok!(SendAsyncMessageResult, {}),
                Err(e) => resp_err!(e),
            }
        },
    )
}

#[named]
fn abi_get_origin_operation_id(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
        store_env,
        arg_offset,
        |handler,
         _req: GetOriginOperationIdRequest|
         -> Result<AbiResponse, WasmV1Error> {
            match handler.interface.get_origin_operation_id() {
                Ok(operation_id) => {
                    resp_ok!(GetOriginOperationIdResult, { operation_id })
                }
                Err(e) => resp_err!(e),
            }
        },
    )
}

#[named]
fn abi_local_execution(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
        store_env,
        arg_offset,
        |handler, req: LocalExecutionRequest| {
            let remaining_gas = handler.get_remaining_gas();
            let module =
                helper_get_module(handler, req.bytecode, remaining_gas)?;

            match crate::execution::run_function(
                handler.interface,
                module,
                &req.target_function_name,
                &req.function_arg,
                remaining_gas,
                handler.get_gas_costs().clone(),
            ) {
                Ok(response) => {
                    handler.set_remaining_gas(response.remaining_gas);
                    resp_ok!(LocalExecutionResponse, { data: response.ret })
                }
                Err(e) => resp_err!(e),
            }
        },
    )
}

#[named]
fn abi_caller_has_write_access(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
        store_env,
        arg_offset,
        |handler,
         _req: CallerHasWriteAccessRequest|
         -> Result<AbiResponse, WasmV1Error> {
            match handler.interface.caller_has_write_access() {
                Ok(has_write_access) => {
                    resp_ok!(CallerHasWriteAccessResult, { has_write_access })
                }
                Err(e) => resp_err!(e),
            }
        },
    )
}

/// Check the exports of a compiled module to see if it contains the given
/// function
#[named]
fn abi_function_exists(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
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

#[named]
fn abi_check_native_amount(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
        store_env,
        arg_offset,
        |handler,
         req: CheckNativeAmountRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let Some(amount) = req.to_check else {
                return resp_err!("No amount to check");
            };

            match handler.interface.check_native_amount_wasmv1(&amount) {
                Ok(is_valid) => {
                    resp_ok!(CheckNativeAmountResult, { is_valid })
                }
                Err(e) => resp_err!(e),
            }
        },
    )
}

#[named]
fn abi_add_native_amount(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
        store_env,
        arg_offset,
        |handler,
         req: AddNativeAmountRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let Some(amount1) = req.amount1 else {
                return resp_err!("No amount1");
            };
            let Some(amount2) = req.amount2 else {
                return resp_err!("No amount2");
            };

            match handler
                .interface
                .add_native_amount_wasmv1(&amount1, &amount2)
            {
                Ok(res) => {
                    resp_ok!(AddNativeAmountResult, { sum: Some(res)})
                }
                Err(e) => resp_err!(e),
            }
        },
    )
}

#[named]
fn abi_sub_native_amount(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
        store_env,
        arg_offset,
        |handler,
         req: SubNativeAmountRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let Some(left) = req.left else {
                return resp_err!("No left amount");
            };
            let Some(right) = req.right else {
                return resp_err!("No right amount");
            };

            match handler.interface.sub_native_amount_wasmv1(&left, &right) {
                Ok(res) => {
                    resp_ok!(SubNativeAmountResult, { difference: Some(res)})
                }
                Err(e) => resp_err!(e),
            }
        },
    )
}

#[named]
fn abi_scalar_mul_native_amount(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
        store_env,
        arg_offset,
        |handler,
         req: ScalarMulNativeAmountRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let Some(amount) = req.amount else {
                return resp_err!("No amount");
            };

            match handler
                .interface
                .scalar_mul_native_amount_wasmv1(&amount, req.coefficient)
            {
                Ok(res) => {
                    resp_ok!(ScalarMulNativeAmountResult, { product: Some(res) })
                }
                Err(e) => resp_err!(e),
            }
        },
    )
}

#[named]
fn abi_scalar_div_rem_native_amount(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
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
                .scalar_div_rem_native_amount_wasmv1(&dividend, req.divisor)
            {
                Ok(res) => {
                    resp_ok!(ScalarDivRemNativeAmountResult,
                        { quotient: Some(res.0), remainder: Some(res.1)})
                }
                Err(e) => resp_err!(e),
            }
        },
    )
}

#[named]
fn abi_div_rem_native_amount(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
        store_env,
        arg_offset,
        |handler,
         req: DivRemNativeAmountRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let Some(dividend) = req.dividend else {
                return resp_err!("No dividend");
            };
            let Some(divisor) = req.divisor else {
                return resp_err!("No divisor");
            };

            match handler
                .interface
                .div_rem_native_amount_wasmv1(&dividend, &divisor)
            {
                Ok(res) => {
                    resp_ok!(DivRemNativeAmountResult,
                        { quotient: res.0, remainder: Some(res.1)})
                }
                Err(e) => resp_err!(e),
            }
        },
    )
}

#[named]
fn abi_native_amount_to_string(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
        store_env,
        arg_offset,
        |handler,
         req: NativeAmountToStringRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let Some(amount) = req.to_convert else {
                return resp_err!("No amount to convert");
            };

            match handler.interface.native_amount_to_string_wasmv1(&amount) {
                Ok(converted_amount) => {
                    resp_ok!(NativeAmountToStringResult, { converted_amount })
                }
                Err(e) => resp_err!(e),
            }
        },
    )
}

#[named]
fn abi_native_amount_from_string(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
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

#[named]
fn abi_base58_check_to_bytes(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
        store_env,
        arg_offset,
        |handler,
         req: Base58CheckToBytesRequest|
         -> Result<AbiResponse, WasmV1Error> {
            match handler
                .interface
                .base58_check_to_bytes_wasmv1(&req.base58_check)
            {
                Ok(bytes) => resp_ok!(Base58CheckToBytesResult, { bytes }),
                Err(e) => resp_err!(e),
            }
        },
    )
}

#[named]
fn abi_bytes_to_base58_check(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
        store_env,
        arg_offset,
        |handler,
         req: BytesToBase58CheckRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let base58_check =
                handler.interface.bytes_to_base58_check_wasmv1(&req.bytes);
            resp_ok!(BytesToBase58CheckResult, { base58_check })
        },
    )
}

#[named]
fn abi_compare_address(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
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
                Err(e) => resp_err!(e),
            }
        },
    )
}

#[named]
fn abi_compare_native_amount(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
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
                Err(e) => resp_err!(e),
            }
        },
    )
}

#[named]
fn abi_compare_native_time(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
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
                Err(e) => resp_err!(e),
            }
        },
    )
}

#[named]
fn abi_compare_pub_key(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
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
                Err(e) => resp_err!(e),
            }
        },
    )
}

#[named]
fn abi_check_address(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
        store_env,
        arg_offset,
        |handler,
         req: CheckAddressRequest|
         -> Result<AbiResponse, WasmV1Error> {
            match handler.interface.check_address_wasmv1(&req.to_check) {
                Ok(is_valid) => {
                    resp_ok!(CheckAddressResult, { is_valid })
                }
                Err(e) => resp_err!(e),
            }
        },
    )
}

#[named]
fn abi_check_pubkey(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
        store_env,
        arg_offset,
        |handler,
         req: CheckPubKeyRequest|
         -> Result<AbiResponse, WasmV1Error> {
            match handler.interface.check_pubkey_wasmv1(&req.to_check) {
                Ok(is_valid) => resp_ok!(CheckPubKeyResult, { is_valid }),
                Err(e) => resp_err!(e),
            }
        },
    )
}

#[named]
fn abi_check_signature(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
        store_env,
        arg_offset,
        |handler, req: CheckSigRequest| -> Result<AbiResponse, WasmV1Error> {
            match handler.interface.check_signature_wasmv1(&req.to_check) {
                Ok(is_valid) => resp_ok!(CheckSigResult, { is_valid }),
                Err(e) => resp_err!(e),
            }
        },
    )
}

#[named]
fn abi_get_address_category(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
        store_env,
        arg_offset,
        |handler,
         req: GetAddressCategoryRequest|
         -> Result<AbiResponse, WasmV1Error> {
            match handler.interface.get_address_category_wasmv1(&req.address) {
                Ok(res) => {
                    resp_ok!(GetAddressCategoryResult, { category: res.into()})
                }
                Err(e) => resp_err!(e),
            }
        },
    )
}

#[named]
fn abi_get_address_version(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
        store_env,
        arg_offset,
        |handler,
         req: GetAddressVersionRequest|
         -> Result<AbiResponse, WasmV1Error> {
            match handler.interface.get_address_version_wasmv1(&req.address) {
                Ok(version) => resp_ok!(GetAddressVersionResult, { version }),
                Err(e) => resp_err!(e),
            }
        },
    )
}

#[named]
fn abi_get_pubkey_version(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
        store_env,
        arg_offset,
        |handler,
         req: GetPubKeyVersionRequest|
         -> Result<AbiResponse, WasmV1Error> {
            match handler.interface.get_pubkey_version_wasmv1(&req.pub_key) {
                Ok(version) => resp_ok!(GetPubKeyVersionResult, { version }),
                Err(e) => resp_err!(e),
            }
        },
    )
}

#[named]
fn abi_get_signature_version(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
        store_env,
        arg_offset,
        |handler,
         req: GetSignatureVersionRequest|
         -> Result<AbiResponse, WasmV1Error> {
            match handler
                .interface
                .get_signature_version_wasmv1(&req.signature)
            {
                Ok(version) => resp_ok!(GetSignatureVersionResult, { version }),
                Err(e) => resp_err!(e),
            }
        },
    )
}

#[named]
fn abi_checked_add_native_time(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
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
                    resp_ok!(CheckedAddNativeTimeResult, { sum: Some(res)})
                }
                Err(e) => resp_err!(e),
            }
        },
    )
}

#[named]
fn abi_checked_sub_native_time(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
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
                    resp_ok!(CheckedSubNativeTimeResult, { difference: Some(res)})
                }
                Err(e) => resp_err!(e),
            }
        },
    )
}

#[named]
fn abi_checked_mul_native_time(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
        store_env,
        arg_offset,
        |handler,
         req: CheckedScalarMulNativeTimeRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let Some(time) = req.time else {
                return resp_err!("No time");
            };

            match handler
                .interface
                .checked_mul_native_time_wasmv1(&time, req.coefficient)
            {
                Ok(res) => {
                    resp_ok!(CheckedScalarMulNativeTimeResult, { product: Some(res)})
                }
                Err(e) => resp_err!(e),
            }
        },
    )
}

#[named]
fn abi_checked_scalar_div_native_time(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
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
                    resp_ok!(CheckedScalarDivRemNativeTimeResult,
                            { quotient: Some(res.0), remainder: Some(res.1)})
                }
                Err(e) => resp_err!(e),
            }
        },
    )
}

#[named]
fn abi_checked_div_native_time(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
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
                    resp_ok!(CheckedDivRemNativeTimeResult,
                            { quotient: res.0, remainder: Some(res.1)})
                }
                Err(e) => resp_err!(e),
            }
        },
    )
}

#[named]
pub fn abi_verify_signature(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        function_name!(),
        store_env,
        arg_offset,
        |handler, req: VerifySigRequest| -> Result<AbiResponse, WasmV1Error> {
            match handler.interface.signature_verify(
                &req.message,
                &req.sig,
                &req.pub_key,
            ) {
                Ok(is_verified) => {
                    resp_ok!(VerifySigResult, { is_verified })
                }
                Err(e) => resp_err!(e),
            }
        },
    )
}
