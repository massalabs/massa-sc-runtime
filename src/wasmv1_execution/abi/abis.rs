use super::{
    super::{env::ABIEnv, WasmV1Error},
    handler::handle_abi,
    proto,
};
use wasmer::{imports, AsStoreMut, Function, FunctionEnv, FunctionEnvMut, Imports};

/// Register all ABIs to a store
pub fn register_abis(store: &mut impl AsStoreMut, shared_abi_env: ABIEnv) -> Imports {
    let fn_env = FunctionEnv::new(store, shared_abi_env);
    imports! {
        "massa" => {
            "abi_abort" =>  Function::new_typed_with_env(store, &fn_env, abi_abort),
            "abi_call" => Function::new_typed_with_env(store, &fn_env, abi_call),
            "abi_create_sc" => Function::new_typed_with_env(store, &fn_env, abi_create_sc),
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
            let Some(proto::Address{address}) = req.address else {
                return Err(WasmV1Error::RuntimeError("No address provided".into()));
            };
            let Some(proto::Amount{amount}) = req.call_coins else {
                return Err(WasmV1Error::RuntimeError("No coins provided".into()));
            };
            let bytecode = handler
                .interface
                .init_call(&address, amount)
                .map_err(|err| {
                    WasmV1Error::RuntimeError(format!("Could not init call: {}", err))
                })?;
            let remaining_gas = handler.get_remaining_gas();
            let module = handler
                .interface
                .get_module(&bytecode, remaining_gas)
                .map_err(|err| {
                    WasmV1Error::RuntimeError(format!("Could not get module: {}", err))
                })?;
            let response = crate::execution::run_function(
                handler.interface,
                module,
                &req.function,
                &req.arg,
                remaining_gas,
                handler.get_gas_costs().clone(),
            )
            .map_err(|err| WasmV1Error::RuntimeError(format!("Could not run function: {}", err)))?;
            handler.set_remaining_gas(response.remaining_gas);
            handler.interface.finish_call().map_err(|err| {
                WasmV1Error::RuntimeError(format!("Could not finish call: {}", err))
            })?;
            Ok(proto::CallResponse {
                return_data: response.ret,
            })
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
            Ok(proto::CreateScResponse {
                address: Some(proto::Address { address }),
            })
        },
    )
}

/// Function designed to abort execution.
pub fn abi_abort(store_env: FunctionEnvMut<ABIEnv>, arg_offset: i32) -> Result<i32, WasmV1Error> {
    handle_abi(
        "abort",
        store_env,
        arg_offset,
        |_handler, req: proto::AbortRequest| -> Result<proto::Empty, WasmV1Error> {
            Err(WasmV1Error::RuntimeError(req.description))
        },
    )
}
