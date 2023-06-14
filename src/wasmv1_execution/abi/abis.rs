use crate::wasmv1_execution::abi::helper_traits::Check;

use super::{
    super::{env::ABIEnv, WasmV1Error},
    handler::{handle_abi, handle_abi_raw},
    helper_traits::{TryInto, TryToU64},
};

use massa_proto_rs::massa::{
    abi::v1::{
        self as proto,
        check_native_address_response::{self, CheckNativeAddressResult},
        native_address_from_string_response::{
            self, NativeAddressFromStringResult,
        },
        native_address_to_string_response::{
            self, NativeAddressToStringResult,
        },
        native_hash_from_string_response::{self, NativeHashFromStringResult},
        native_hash_to_string_response::{self, NativeHashToStringResult},
        native_pub_key_from_string_response::{
            self, NativePubKeyFromStringResult,
        },
        native_pub_key_to_string_response::{self, NativePubKeyToStringResult},
        native_sig_from_string_response::{self, NativeSigFromStringResult},
        native_sig_to_string_response::{self, NativeSigToStringResult},
        AddNativeAmountsRequest, AddNativeAmountsResponse, CallRequest,
        CallResponse, CheckNativeAddressRequest, CheckNativeAddressResponse,
        CheckNativeAmountRequest, CheckNativeAmountResponse,
        CheckNativeHashRequest, CheckNativeHashResponse,
        CheckNativePubKeyRequest, CheckNativePubKeyResponse,
        CheckNativeSigRequest, CheckNativeSigResponse, CreateScRequest,
        CreateScResponse, DivRemNativeAmountRequest,
        DivRemNativeAmountResponse, Empty, FunctionExistsRequest,
        FunctionExistsResponse, GenerateEventRequest, LocalCallRequest,
        LocalCallResponse, LogRequest, MulNativeAmountRequest,
        MulNativeAmountResponse, NativeAddressFromStringRequest,
        NativeAddressFromStringResponse, NativeAddressToStringRequest,
        NativeAddressToStringResponse, NativeAmountFromBytesRequest,
        NativeAmountFromBytesResponse, NativeAmountFromStringRequest,
        NativeAmountFromStringResponse, NativeAmountToBytesRequest,
        NativeAmountToBytesResponse, NativeAmountToStringRequest,
        NativeAmountToStringResponse, NativeHashFromStringRequest,
        NativeHashFromStringResponse, NativeHashToStringRequest,
        NativeHashToStringResponse, NativePubKeyFromStringRequest,
        NativePubKeyFromStringResponse, NativePubKeyToStringRequest,
        NativePubKeyToStringResponse, NativeSigFromStringRequest,
        NativeSigFromStringResponse, NativeSigToStringRequest,
        NativeSigToStringResponse, SubNativeAmountsRequest,
        SubNativeAmountsResponse, TransferCoinsRequest, check_native_pub_key_response::{self, CheckNativePubKeyResult}, check_native_sig_response::{self, CheckNativeSigResult},
    },
    model::v1::{AddressCategory, NativeAddress, NativePubKey},
};

use wasmer::{
    imports, AsStoreMut, Function, FunctionEnv, FunctionEnvMut, Imports,
};

// This macro ease the construction of the Error variant of the response to an
// ABI call.
// /!\ it requires `Rest` to be aliased to the correct response type.
// /!\ this is achieved by a `use whatever_mod_name::Resp as Resp` at the
// /!\ beginning of the abi function.
macro_rules! resp_err {
    ($err:expr) => {
        Resp::Error(proto::Error {
            message: $err.to_string(),
        })
    };
}

// This macro is used to construct a response to an ABI call.
// It is used in the abi_* functions below.
// /!\ for this macro to work 'Response' must be in scope.
// /!\ as every response have a difference name but the same structure this is
// /!\ achieved by a `use WhatEverResponse as Response` at the beginning of the
// /!à abi function.
macro_rules! response {
    ($response:expr) => {
        Response {
            resp: Some($response),
        }
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
            "abi_native_amount_to_bytes" => Function::new_typed_with_env(
                store,
                &fn_env,
                abi_native_amount_to_bytes,
            ),
            "abi_native_amount_from_bytes" => Function::new_typed_with_env(
                store,
                &fn_env,
                abi_native_amount_from_bytes,
            ),


            "abi_log" => Function::new_typed_with_env(store, &fn_env, abi_log),
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
                .init_call(&TryInto::try_into(&address)?, amount.try_to_u64()?)
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
        |handler, req: LocalCallRequest| {
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

            Ok(LocalCallResponse { data: response.ret })
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
        |handler, req: CreateScRequest| {
            let address = handler
                .interface
                .create_module(&req.bytecode)
                .map_err(|err| {
                    WasmV1Error::RuntimeError(format!(
                        "Could not create new smart contract: {}",
                        err
                    ))
                })?;

            tracing::warn!("FIXME: NativeAddress version is hardcoded to 0");
            Ok(CreateScResponse {
                sc_address: Some(NativeAddress {
                    category: AddressCategory::ScAddress as i32,
                    version: 0u64,
                    content: address.into_bytes(),
                }),
            })
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
        |handler, req: TransferCoinsRequest| -> Result<Empty, WasmV1Error> {
            let address = req.target_address.ok_or(
                WasmV1Error::RuntimeError("No address provided".into()),
            )?;
            let amount = req
                .amount_to_transfer
                .ok_or(WasmV1Error::RuntimeError("No coins provided".into()))?;

            // Do not remove this. It could be used for gas_calibration in
            // future. if cfg!(feature = "gas_calibration") {
            //     let fname = format!("massa.{}:0", function_name!());
            //     param_size_update(&env, &mut ctx, &fname, to_address.len(),
            // true); }

            handler
                .interface
                .transfer_coins(
                    &TryInto::try_into(&address)?,
                    amount.try_to_u64()?,
                )
                .map_err(|err| {
                    WasmV1Error::RuntimeError(format!(
                        "Transfer coins failed: {}",
                        err
                    ))
                })?;

            Ok(Empty {})
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
        |_handler, req: GenerateEventRequest| {
            _handler
                .interface
                .generate_event(req.event)
                .map_err(|err| {
                    WasmV1Error::RuntimeError(format!(
                        "Failed to generate event: {}",
                        err
                    ))
                })?;

            Ok(Empty {})
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
         -> Result<FunctionExistsResponse, WasmV1Error> {
            let address = req.target_sc_address.ok_or_else(|| {
                WasmV1Error::RuntimeError("No address provided".into())
            })?;

            let bytecode =
                helper_get_bytecode(handler, TryInto::try_into(&address)?)?;

            let remaining_gas = if cfg!(feature = "gas_calibration") {
                u64::MAX
            } else {
                handler.get_remaining_gas()
            };

            Ok(FunctionExistsResponse {
                exists: helper_get_module(handler, bytecode, remaining_gas)?
                    .function_exists(&req.function_name),
            })
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

pub fn abi_log(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi("log", store_env, arg_offset, |_handler, req: LogRequest| {
        let message = req.message;

        println!("wasm log: {}", message);

        Ok(Empty {})
    })
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
         -> Result<NativeAddressToStringResponse, WasmV1Error> {
            use native_address_to_string_response::Resp;
            use NativeAddressToStringResponse as Response;

            let Some(address) = req.to_convert else {
                return Ok(response!(resp_err!("No address to convert")));
            };

            let resp = match TryInto::try_into(&address) {
                Ok(addr) => Resp::Res(NativeAddressToStringResult {
                    converted_address: addr,
                }),
                Err(err) => resp_err!(err),
            };

            Ok(response!(resp))
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
         -> Result<NativePubKeyToStringResponse, WasmV1Error> {
            use native_pub_key_to_string_response::Resp;
            use NativePubKeyToStringResponse as Response;

            let Some(pubkey) = req.to_convert else {
            return Ok(response!(resp_err!("No pubkey to convert")));
            };

            let resp = match TryInto::try_into(&pubkey) {
                Ok(pubkey) => Resp::Res(NativePubKeyToStringResult {
                    converted_pubkey: pubkey,
                }),
                Err(err) => resp_err!(err),
            };

            Ok(response!(resp))
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
         -> Result<NativeSigToStringResponse, WasmV1Error> {
            use native_sig_to_string_response::Resp;
            use NativeSigToStringResponse as Response;

            let Some(sig) = req.to_convert else {
                return Ok(response!(resp_err!("No sig to convert")));

            };

            let resp = match TryInto::try_into(&sig) {
                Ok(sig) => {
                    Resp::Res(NativeSigToStringResult { converted_sig: sig })
                }
                Err(err) => resp_err!(err),
            };

            Ok(response!(resp))
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
         -> Result<NativeHashToStringResponse, WasmV1Error> {
            use native_hash_to_string_response::Resp;
            use NativeHashToStringResponse as Response;

            let Some(hash) = req.to_convert else {
                return Ok(response!(resp_err!("No hash to convert")));
            };

            let resp = match TryInto::try_into(&hash) {
                Ok(hash) => Resp::Res(NativeHashToStringResult {
                    converted_hash: hash,
                }),
                Err(err) => resp_err!(err),
            };

            Ok(response!(resp))
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
         -> Result<NativeAddressFromStringResponse, WasmV1Error> {
            use native_address_from_string_response::Resp;
            use NativeAddressFromStringResponse as Response;

            let resp = match TryInto::try_into(&req.to_convert) {
                Ok(address) => Resp::Res(NativeAddressFromStringResult {
                    converted_address: Some(address),
                }),
                Err(err) => resp_err!(err),
            };

            Ok(response!(resp))
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
         -> Result<NativePubKeyFromStringResponse, WasmV1Error> {
            use native_pub_key_from_string_response::Resp;
            use NativePubKeyFromStringResponse as Response;

            let resp = match TryInto::try_into(&req.to_convert) {
                Ok(pubkey) => Resp::Res(NativePubKeyFromStringResult {
                    converted_pubkey: Some(pubkey),
                }),
                Err(err) => resp_err!(err),
            };

            Ok(response!(resp))
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
         -> Result<NativeSigFromStringResponse, WasmV1Error> {
            use native_sig_from_string_response::Resp;
            use NativeSigFromStringResponse as Response;

            let resp = match TryInto::try_into(&req.to_convert) {
                Ok(sig) => Resp::Res(NativeSigFromStringResult {
                    converted_sig: Some(sig),
                }),
                Err(err) => resp_err!(err),
            };

            Ok(response!(resp))
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
         -> Result<NativeHashFromStringResponse, WasmV1Error> {
            use native_hash_from_string_response::Resp;
            use NativeHashFromStringResponse as Response;

            let resp = match TryInto::try_into(&req.to_convert) {
                Ok(hash) => Resp::Res(NativeHashFromStringResult {
                    converted_hash: Some(hash),
                }),
                Err(err) => resp_err!(err),
            };

            Ok(response!(resp))
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
         -> Result<CheckNativeAddressResponse, WasmV1Error> {
            use check_native_address_response::Resp;
            use CheckNativeAddressResponse as Response;

            let Some(address) = req.to_check else {
                return Ok(response!(resp_err!("No address to check")));
            };

            let resp = match address.is_valid() {
                Ok(is_valid) => {
                    Resp::Res(CheckNativeAddressResult { is_valid })
                }
                Err(err) => resp_err!(err),
            };

            Ok(response!(resp))
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
         -> Result<CheckNativePubKeyResponse, WasmV1Error> {
            use check_native_pub_key_response::Resp;
            use CheckNativePubKeyResponse as Response;

            let Some(pubkey) = req.to_check else {
                return Ok(response!(resp_err!("No pubkey to check")));
             };

            let resp = match pubkey.is_valid() {
                Ok(is_valid) => Resp::Res(CheckNativePubKeyResult { is_valid }),
                Err(err) => resp_err!(err),
            };

            Ok(response!(resp))
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
         -> Result<CheckNativeSigResponse, WasmV1Error> {
            use check_native_sig_response::Resp;
            use CheckNativeSigResponse as Response;

            let Some(sig) = req.to_check else {
                return Ok(response!(resp_err!("No sig to check")));
            };

            let resp = match sig.is_valid() {
                Ok(is_valid) => Resp::Res(CheckNativeSigResult { is_valid }),
                Err(err) => resp_err!(err),
            };

            Ok(response!(resp))

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
         -> Result<CheckNativeHashResponse, WasmV1Error> { todo!() },
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
         -> Result<CheckNativeAmountResponse, WasmV1Error> { todo!() },
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
         -> Result<AddNativeAmountsResponse, WasmV1Error> { todo!() },
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
         -> Result<SubNativeAmountsResponse, WasmV1Error> { todo!() },
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
         -> Result<MulNativeAmountResponse, WasmV1Error> { todo!() },
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
         -> Result<DivRemNativeAmountResponse, WasmV1Error> { todo!() },
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
         -> Result<DivRemNativeAmountResponse, WasmV1Error> { todo!() },
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
         -> Result<NativeAmountToStringResponse, WasmV1Error> {
            todo!()
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
        |_handler,
         req: NativeAmountFromStringRequest|
         -> Result<NativeAmountFromStringResponse, WasmV1Error> {
            todo!()
        },
    )
}
pub fn abi_native_amount_to_bytes(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "native_amount_to_bytes",
        store_env,
        arg_offset,
        |_handler,
         req: NativeAmountToBytesRequest|
         -> Result<NativeAmountToBytesResponse, WasmV1Error> {
            todo!()
        },
    )
}
pub fn abi_native_amount_from_bytes(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(
        "native_amount_from_bytes",
        store_env,
        arg_offset,
        |_handler,
         req: NativeAmountFromBytesRequest|
         -> Result<NativeAmountFromBytesResponse, WasmV1Error> {
            todo!()
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
