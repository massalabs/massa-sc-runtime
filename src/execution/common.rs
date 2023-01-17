use displaydoc::Display;
use thiserror::Error;
use wasmer::FunctionEnvMut;

use crate::env::{get_remaining_points, set_remaining_points, ASEnv, MassaEnv};
use crate::Response;

use super::{init_store, ASContextModule};

pub(crate) type ABIResult<T, E = ABIError> = core::result::Result<T, E>;

#[derive(Display, Error, Debug)]
pub enum ABIError {
    /// Runtime error: {0}
    Error(#[from] anyhow::Error),
    /// Wasmer runtime error: {0}
    RuntimeError(#[from] wasmer::RuntimeError),
    /// Wasmer compile error: {0}
    CompileError(#[from] wasmer::CompileError),
    /// Wasmer instantiation error: {0}
    InstantiationError(#[from] wasmer::InstantiationError),
    /// Runtime serde_json error: {0}
    SerdeError(#[from] serde_json::Error),
}

macro_rules! abi_bail {
    ($err:expr) => {
        return Err(crate::execution::ABIError::Error(anyhow::anyhow!(
            $err.to_string()
        )))
    };
}

pub(crate) use abi_bail;

/// `Call` ABI called by the webassembly VM
///
/// Call an exported function in a WASM module at a given address
///
/// It take in argument the environment defined in env.rs
/// this environment is automatically filled by the wasmer library
/// And two pointers of string. (look at the readme in the wasm folder)
pub(crate) fn call_module(
    ctx: &mut FunctionEnvMut<ASEnv>,
    address: &str,
    function: &str,
    param: &[u8],
    raw_coins: i64,
) -> ABIResult<Response> {
    let raw_coins: u64 = match raw_coins.try_into() {
        Ok(v) => v,
        Err(_) => abi_bail!("negative amount of coins in Call"),
    };
    let env = ctx.data().clone();
    let bytecode = env.get_interface().init_call(address, raw_coins)?;
    let interface = env.get_interface();
    let gas_costs = env.get_gas_costs();

    let remaining_gas = if cfg!(feature = "gas_calibration") {
        u64::MAX
    } else {
        get_remaining_points(&env, ctx)?
    };

    let module = interface.get_module(&bytecode, remaining_gas, gas_costs.clone())?;
    let resp = crate::execution_impl::exec(&*interface, module, function, param, gas_costs)?;
    if cfg!(not(feature = "gas_calibration")) {
        set_remaining_points(&env, ctx, resp.0.remaining_gas)?;
    }
    env.get_interface().finish_call()?;
    Ok(resp.0)
}

/// Alternative to `call_module` to execute bytecode in a local context
pub(crate) fn local_call(
    ctx: &mut FunctionEnvMut<ASEnv>,
    bytecode: &[u8],
    function: &str,
    param: &[u8],
) -> ABIResult<Response> {
    let env = ctx.data().clone();
    let interface = env.get_interface();
    let gas_costs = env.get_gas_costs();

    let remaining_gas = if cfg!(feature = "gas_calibration") {
        u64::MAX
    } else {
        get_remaining_points(&env, ctx)?
    };

    let module = interface.get_module(&bytecode, remaining_gas, gas_costs.clone())?;
    let resp = crate::execution_impl::exec(&*interface, module, function, param, gas_costs)?;
    if cfg!(not(feature = "gas_calibration")) {
        set_remaining_points(&env, ctx, resp.0.remaining_gas)?;
    }
    Ok(resp.0)
}

pub(crate) fn function_exists(
    ctx: &mut FunctionEnvMut<ASEnv>,
    address: &str,
    function: &str,
) -> ABIResult<bool> {
    let env = ctx.data().clone();
    let interface = env.get_interface();
    let bytecode = interface.raw_get_bytecode_for(address)?;
    let gas_costs = env.get_gas_costs();

    let remaining_gas = if cfg!(feature = "gas_calibration") {
        u64::MAX
    } else {
        get_remaining_points(&env, ctx)?
    };

    let module = interface.get_module(&bytecode, remaining_gas, gas_costs.clone())?;
    let mut store = init_store(&module.0.engine)?;
    let mut context_module = ASContextModule::new(&*interface, module.0.binary_module, gas_costs);
    let instance = context_module.create_vm_instance_and_init_env(&mut store)?;

    Ok(instance.exports.get_function(function).is_ok())
}

/// Create a smart contract with the given `bytecode`
pub(crate) fn create_sc(ctx: &mut FunctionEnvMut<ASEnv>, bytecode: &[u8]) -> ABIResult<String> {
    let env = ctx.data();
    Ok(env.get_interface().create_module(bytecode)?)
}
