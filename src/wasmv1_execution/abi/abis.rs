use crate::wasmv1_execution::abi::helper_traits::Check;

use super::{
    super::{env::ABIEnv, WasmV1Error},
    handler::{handle_abi, handle_abi_raw},
    helper_traits::TryInto,
};

use massa_proto_rs::massa::{
    abi::v1::{
        self as proto, abi_response, resp_result, AbiResponse,
        AddNativeAmountsRequest, AppendDataRequest, AppendDataResult,
        CallRequest, CallResponse, CheckNativeAddressRequest,
        CheckNativeAddressResult, CheckNativeAmountRequest,
        CheckNativeHashRequest, CheckNativePubKeyRequest,
        CheckNativePubKeyResult, CheckNativeSigRequest, CheckNativeSigResult,
        CreateScRequest, CreateScResult, DeleteDataRequest, DeleteDataResult,
        DivRemNativeAmountRequest, FunctionExistsRequest, FunctionExistsResult,
        GenerateEventRequest, GenerateEventResult, GetCurrentPeriodRequest,
        GetCurrentPeriodResult, GetCurrentThreadRequest,
        GetCurrentThreadResult, GetDataRequest, GetDataResult, HasDataRequest,
        HasDataResult, MulNativeAmountRequest, NativeAddressFromStringRequest,
        NativeAddressFromStringResult, NativeAddressToStringRequest,
        NativeAddressToStringResult, NativeAmountFromStringRequest,
        NativeAmountToStringRequest, NativeHashFromStringRequest,
        NativeHashFromStringResult, NativeHashToStringRequest,
        NativeHashToStringResult, NativePubKeyFromStringRequest,
        NativePubKeyFromStringResult, NativePubKeyToStringRequest,
        NativePubKeyToStringResult, NativeSigFromStringRequest,
        NativeSigFromStringResult, NativeSigToStringRequest,
        NativeSigToStringResult, RespResult, SetDataRequest, SetDataResult,
        SubNativeAmountsRequest, TransferCoinsRequest, TransferCoinsResult, NativeHashRequest, HashSha256Request, HashSha256Result, NativeHashResult,
    },
    model::v1::{AddressCategory, NativeAddress, NativePubKey},
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

            "abi_native_address_to_string" => Function::new_typed_with_env(
                store,
                &fn_env,
                abi_native_address_to_string,
            ),
            "abi_native_pubkey_to_string" => Function::new_typed_with_env(
                store,
                &fn_env,
                abi_native_pubkey_to_string,
            ),
            "abi_native_sig_to_string" => Function::new_typed_with_env(
                store,
                &fn_env,
                abi_native_sig_to_string,
            ),
            "abi_native_hash_to_string" => Function::new_typed_with_env(
                store,
                &fn_env,
                abi_native_hash_to_string,
            ),

            "abi_native_address_from_string" => Function::new_typed_with_env(
                store,
                &fn_env,
                abi_native_address_from_string,
            ),
            "abi_native_pubkey_from_string" => Function::new_typed_with_env(
                store,
                &fn_env,
                abi_native_pubkey_from_string,
            ),
            "abi_native_sig_from_string" => Function::new_typed_with_env(
                store,
                &fn_env,
                abi_native_sig_from_string,
            ),
            "abi_native_hash_from_string" => Function::new_typed_with_env(
                store,
                &fn_env,
                abi_native_hash_from_string,
            ),

            "abi_check_native_address" => Function::new_typed_with_env(
                store,
                &fn_env,
                abi_check_native_address,
            ),
            "abi_check_native_pubkey" => Function::new_typed_with_env(
                store,
                &fn_env,
                abi_check_native_pubkey,
            ),
            "abi_check_native_sig" =>
                Function::new_typed_with_env(store, &fn_env, abi_check_native_sig),

            "abi_check_native_hash" =>
                Function::new_typed_with_env(store, &fn_env, abi_check_native_hash),

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
            "abi_get_current_period" => Function::new_typed_with_env(
                store,
                &fn_env,
                abi_get_current_period,
            ),
            "abi_get_current_thread" => Function::new_typed_with_env(
                store,
                &fn_env,
                abi_get_current_thread,
            ),
            "abi_native_hash" => Function::new_typed_with_env(
                store,
                &fn_env,
                abi_native_hash,
            ),
            "abi_hash_sha256" => Function::new_typed_with_env(
                store,
                &fn_env,
                abi_hash_sha256,
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
            let address = req.target_sc_address.ok_or_else(|| {
                WasmV1Error::RuntimeError("No address provided".into())
            })?;

            let amount = req.call_coins.ok_or_else(|| {
                WasmV1Error::RuntimeError("No coins provided".into())
            })?;

            let bytecode = handler
                .interface
                .init_call(
                    &TryInto::try_into(&address)?,
                    TryInto::try_into(&amount)?,
                )
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
            let address = req.target_sc_address.ok_or_else(|| {
                WasmV1Error::RuntimeError("No address provided".into())
            })?;

            let bytecode =
                helper_get_bytecode(handler, TryInto::try_into(&address)?)?;
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
            match handler.interface.create_module(&req.bytecode) {
                Ok(addr) => {
                    tracing::warn!(
                        "FIXME: NativeAddress version is hardcoded to 0"
                    );
                    let addr = NativeAddress {
                        category: AddressCategory::ScAddress as i32,
                        version: 0u64,
                        content: addr.into_bytes(),
                    };
                    resp_ok!(CreateScResult, {
                        sc_address: Some(addr)
                    })
                }
                Err(err) => resp_err!(err),
            }
        },
    )
}

/// gets the period of the current execution slot
pub(crate) fn abi_get_current_period(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "get_current_period",
        store_env,
        arg_offset,
        |handler,
         _req: GetCurrentPeriodRequest|
         -> Result<AbiResponse, WasmV1Error> {
            // Do not remove this. It could be used for gas_calibration in
            // future. if cfg!(feature = "gas_calibration") {
            //     let fname = format!("massa.{}:0", function_name!());
            //     param_size_update(&env, &mut ctx, &fname, to_address.len(),
            // true); }

            match handler.interface.get_current_period() {
                Ok(period) => resp_ok!(GetCurrentPeriodResult, {
                    period: period as i64
                }),
                Err(err) => resp_err!(err),
            }
        },
    )
}

/// gets the thread of the current execution slot
pub(crate) fn abi_get_current_thread(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "get_current_thread",
        store_env,
        arg_offset,
        |handler,
         _req: GetCurrentThreadRequest|
         -> Result<AbiResponse, WasmV1Error> {
            // Do not remove this. It could be used for gas_calibration in
            // future. if cfg!(feature = "gas_calibration") {
            //     let fname = format!("massa.{}:0", function_name!());
            //     param_size_update(&env, &mut ctx, &fname, to_address.len(),
            // true); }

            match handler.interface.get_current_thread() {
                Ok(thread) => resp_ok!(GetCurrentThreadResult, {
                    thread: thread as i32
                }),
                Err(err) => resp_err!(err),
            }
        },
    )
}

/// performs a hash on a bytearray and returns the native_hash
pub(crate) fn abi_native_hash(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "native_hash",
        store_env,
        arg_offset,
        |handler,
        req: NativeHashRequest|
         -> Result<AbiResponse, WasmV1Error> {
            // Do not remove this. It could be used for gas_calibration in
            // future. if cfg!(feature = "gas_calibration") {
            //     let fname = format!("massa.{}:0", function_name!());
            //     param_size_update(&env, &mut ctx, &fname, to_address.len(),
            // true); }
            
            match handler.interface.native_hash(&req.data) {
                Ok(hash) => {
                    resp_ok!(NativeHashResult, { hash: Some(hash) })
                },
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
        |handler,
        req: HashSha256Request|
         -> Result<AbiResponse, WasmV1Error> {
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
            let Some(address) = req.target_address else {
                return resp_err!("No address provided");
            };

            let Some(amount) = req.amount_to_transfer else {
                return resp_err!("No coins provided");
            };

            // Do not remove this. It could be used for gas_calibration in
            // future. if cfg!(feature = "gas_calibration") {
            //     let fname = format!("massa.{}:0", function_name!());
            //     param_size_update(&env, &mut ctx, &fname, to_address.len(),
            // true); }

            let Ok(address) = TryInto::try_into(&address) else {
                return resp_err!("Invalid address");
            };

            let Ok(raw_amount) = TryInto::try_into( &amount) else {
                return resp_err!("Invalid amount");
            };

            let transfer_coins =
                handler.interface.transfer_coins(&address, raw_amount);
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
         -> Result<GenerateEventResult, WasmV1Error> {
            _handler
                .interface
                .generate_event(req.event)
                .map_err(|err| {
                    WasmV1Error::RuntimeError(format!(
                        "Failed to generate event: {}",
                        err
                    ))
                })?;

            Ok(GenerateEventResult {})
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
            if let Err(e) = handler.interface.raw_set_data(&req.key, &req.value)
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
            let Ok(data) = handler.interface.raw_get_data(&req.key) else
            {
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
            if let Err(e) = handler.interface.raw_delete_data(&req.key) {
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
            if let Err(e) =
                handler.interface.raw_append_data(&req.key, &req.value)
            {
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
            let Ok(res) = handler.interface.has_data(&req.key) else
            {
                return resp_err!("Failed to check if data exists");
            };
            resp_ok!(HasDataResult, { has_data: res })
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
            let Some(address) = req.target_sc_address else {
                return resp_err!("No address provided");
            };

            let Ok(address) = TryInto::try_into(&address) else {
                return resp_err!("Invalid address");
            };

            let Ok(bytecode) = helper_get_bytecode(handler, address) else {
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

pub fn abi_native_address_to_string(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "native_address_to_string",
        store_env,
        arg_offset,
        |_handler,
         req: NativeAddressToStringRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let Some(address) = req.to_convert else {
                return resp_err!("No address to convert");
            };

            match TryInto::try_into(&address) {
                Ok(addr) => resp_ok!(NativeAddressToStringResult, {
                    converted_address: addr,
                }),
                Err(err) => resp_err!(err),
            }
        },
    )
}

pub fn abi_native_pubkey_to_string(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "native_pubkey_to_string",
        store_env,
        arg_offset,
        |_handler,
         req: NativePubKeyToStringRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let Some(pubkey) = req.to_convert else {
                return resp_err!("No pubkey to convert");
            };

            match TryInto::try_into(&pubkey) {
                Ok(pubkey) => resp_ok!(NativePubKeyToStringResult, {
                    converted_pubkey: pubkey,
                }),
                Err(err) => resp_err!(err),
            }
        },
    )
}

pub fn abi_native_sig_to_string(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "native_sig_to_string",
        store_env,
        arg_offset,
        |_handler,
         req: NativeSigToStringRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let Some(sig) = req.to_convert else {
                return resp_err!("No sig to convert");

            };

            match TryInto::try_into(&sig) {
                Ok(sig) => {
                    resp_ok!(NativeSigToStringResult, { converted_sig: sig })
                }
                Err(err) => resp_err!(err),
            }
        },
    )
}

pub fn abi_native_hash_to_string(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "native_hash_to_string",
        store_env,
        arg_offset,
        |_handler,
         req: NativeHashToStringRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let Some(hash) = req.to_convert else {
                return resp_err!("No hash to convert");
            };

            match TryInto::try_into(&hash) {
                Ok(hash) => resp_ok!(NativeHashToStringResult, {
                    converted_hash: hash,
                }),
                Err(err) => resp_err!(err),
            }
        },
    )
}

pub fn abi_native_address_from_string(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "native_address_from_string",
        store_env,
        arg_offset,
        |_handler,
         req: NativeAddressFromStringRequest|
         -> Result<AbiResponse, WasmV1Error> {
            match TryInto::try_into(&req.to_convert) {
                Ok(address) => resp_ok!(NativeAddressFromStringResult, {
                    converted_address: Some(address),
                }),
                Err(err) => resp_err!(err),
            }
        },
    )
}

pub fn abi_native_pubkey_from_string(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "native_pubkey_from_string",
        store_env,
        arg_offset,
        |_handler,
         req: NativePubKeyFromStringRequest|
         -> Result<AbiResponse, WasmV1Error> {
            match TryInto::try_into(&req.to_convert) {
                Ok(pubkey) => resp_ok!(NativePubKeyFromStringResult, {
                    converted_pubkey: Some(pubkey),
                }),
                Err(err) => resp_err!(err),
            }
        },
    )
}

pub fn abi_native_sig_from_string(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "native_sig_from_string",
        store_env,
        arg_offset,
        |_handler,
         req: NativeSigFromStringRequest|
         -> Result<AbiResponse, WasmV1Error> {
            match TryInto::try_into(&req.to_convert) {
                Ok(sig) => resp_ok!(NativeSigFromStringResult, {
                    converted_sig: Some(sig),
                }),
                Err(err) => resp_err!(err),
            }
        },
    )
}

pub fn abi_native_hash_from_string(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "native_hash_from_string",
        store_env,
        arg_offset,
        |_handler,
         req: NativeHashFromStringRequest|
         -> Result<AbiResponse, WasmV1Error> {
            match TryInto::try_into(&req.to_convert) {
                Ok(hash) => resp_ok!(NativeHashFromStringResult, {
                    converted_hash: Some(hash),
                }),
                Err(err) => resp_err!(err),
            }
        },
    )
}

pub fn abi_check_native_address(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "check_native_address",
        store_env,
        arg_offset,
        |_handler,
         req: CheckNativeAddressRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let Some(address) = req.to_check else {
                return resp_err!("No address to check");
            };

            match address.is_valid() {
                Ok(is_valid) => {
                    resp_ok!(CheckNativeAddressResult, { is_valid: is_valid })
                }
                Err(err) => resp_err!(err),
            }
        },
    )
}
pub fn abi_check_native_pubkey(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "check_native_pubkey",
        store_env,
        arg_offset,
        |_handler,
         req: CheckNativePubKeyRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let Some(pubkey) = req.to_check else {
                return resp_err!("No pubkey to check");
             };

            match pubkey.is_valid() {
                Ok(is_valid) => {
                    resp_ok!(CheckNativePubKeyResult, { is_valid: is_valid

                    })
                }
                Err(err) => resp_err!(err),
            }
        },
    )
}
pub fn abi_check_native_sig(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "check_native_sig",
        store_env,
        arg_offset,
        |_handler,
         req: CheckNativeSigRequest|
         -> Result<AbiResponse, WasmV1Error> {
            let Some(sig) = req.to_check else {
                return resp_err!("No sig to check");
            };

            match sig.is_valid() {
                Ok(is_valid) => {
                    resp_ok!(CheckNativeSigResult, { is_valid: is_valid })
                }
                Err(err) => resp_err!(err),
            }
        },
    )
}
pub fn abi_check_native_hash(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "check_native_hash",
        store_env,
        arg_offset,
        |_handler,
         req: CheckNativeHashRequest|
         -> Result<AbiResponse, WasmV1Error> { todo!() },
    )
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
        |_handler,
         req: NativeAmountToStringRequest|
         -> Result<AbiResponse, WasmV1Error> { todo!() },
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
        |_handler,
         req: NativeAmountFromStringRequest|
         -> Result<AbiResponse, WasmV1Error> { todo!() },
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
