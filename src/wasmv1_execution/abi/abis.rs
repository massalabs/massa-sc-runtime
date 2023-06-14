use crate::wasmv1_execution::types::{TryToString, TryToU64};

use super::{
    super::{env::ABIEnv, WasmV1Error},
    handler::{handle_abi, handle_abi_raw},
};
use massa_proto_rs::massa::{
    abi::v1::{self as proto, NativeAddressToStringRequest},
    model::v1::{AddressCategory, NativeAddress},
};

use wasmer::{
    imports, AsStoreMut, Function, FunctionEnv, FunctionEnvMut, Imports,
};

/// Register all ABIs to a store
pub fn register_abis(
    store: &mut impl AsStoreMut,
    shared_abi_env: ABIEnv,
) -> Imports {
    let fn_env = FunctionEnv::new(store, shared_abi_env);
    imports! {
        "massa" => {
            "abi_abort" =>  Function::new_typed_with_env(store, &fn_env, abi_abort),
            "abi_call" => Function::new_typed_with_env(store, &fn_env, abi_call),
            "abi_local_call" => Function::new_typed_with_env(store, &fn_env, abi_local_call),
            "abi_create_sc" => Function::new_typed_with_env(store, &fn_env, abi_create_sc),
            "abi_transfer_coins" => Function::new_typed_with_env(store, &fn_env, abi_transfer_coins),
            "abi_generate_event" => Function::new_typed_with_env(store, &fn_env, abi_generate_event),
            "abi_function_exists" => Function::new_typed_with_env(store, &fn_env, abi_function_exists),
            "abi_native_address_to_string" =>
                Function::new_typed_with_env(store, &fn_env, abi_native_address_to_string),

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
        |handler, req: proto::CallRequest| {
            let address = req.target_sc_address.ok_or_else(|| {
                WasmV1Error::RuntimeError("No address provided".into())
            })?;

            let amount = req.call_coins.ok_or_else(|| {
                WasmV1Error::RuntimeError("No coins provided".into())
            })?;

            let bytecode = handler
                .interface
                .init_call(&address.try_to_string()?, amount.try_to_u64()?)
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
            Ok(proto::CallResponse { data: response.ret })
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
        |handler, req: proto::LocalCallRequest| {
            let address = req.target_sc_address.ok_or_else(|| {
                WasmV1Error::RuntimeError("No address provided".into())
            })?;

            let bytecode =
                helper_get_bytecode(handler, address.try_to_string()?)?;
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

            Ok(proto::LocalCallResponse { data: response.ret })
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
        |handler, req: proto::CreateScRequest| {
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
            Ok(proto::CreateScResponse {
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
        |handler,
         req: proto::TransferCoinsRequest|
         -> Result<proto::Empty, WasmV1Error> {
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
                .transfer_coins(&address.try_to_string()?, amount.try_to_u64()?)
                .map_err(|err| {
                    WasmV1Error::RuntimeError(format!(
                        "Transfer coins failed: {}",
                        err
                    ))
                })?;

            Ok(proto::Empty {})
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
        |_handler, req: proto::GenerateEventRequest| {
            _handler
                .interface
                .generate_event(req.event)
                .map_err(|err| {
                    WasmV1Error::RuntimeError(format!(
                        "Failed to generate event: {}",
                        err
                    ))
                })?;

            Ok(proto::Empty {})
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
         req: proto::FunctionExistsRequest|
         -> Result<proto::FunctionExistsResponse, WasmV1Error> {
            let address = req.target_sc_address.ok_or_else(|| {
                WasmV1Error::RuntimeError("No address provided".into())
            })?;

            let bytecode =
                helper_get_bytecode(handler, address.try_to_string()?)?;

            let remaining_gas = if cfg!(feature = "gas_calibration") {
                u64::MAX
            } else {
                handler.get_remaining_gas()
            };

            Ok(proto::FunctionExistsResponse {
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
    handle_abi(
        "log",
        store_env,
        arg_offset,
        |_handler, req: proto::LogRequest| {
            let message = req.message;

            println!("wasm log: {}", message);

            Ok(proto::Empty {})
        },
    )
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
         -> Result<Vec<u8>, WasmV1Error> {
            let address = req.address_to_convert.ok_or_else(|| {
                WasmV1Error::RuntimeError("No address to convert".to_string())
            })?;

            let address = NativeAddress::try_from(address).map_err(|err| {
                WasmV1Error::RuntimeError(format!(
                    "Could not convert to address: {}",
                    err
                ))
            })?;

            Ok(address.try_to_string()?.into_bytes())
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
