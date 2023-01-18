use wasmer::FunctionEnvMut;

use crate::env::{get_remaining_points, set_remaining_points, ASEnv, MassaEnv};
use crate::{Response, RuntimeModule};

use super::abi_error::{abi_bail, ABIResult};

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

    let remaining_gas = if cfg!(feature = "gas_calibration") {
        u64::MAX
    } else {
        get_remaining_points(&env, ctx)?
    };

    let module = interface.get_module(&bytecode, remaining_gas)?;
    let resp = crate::execution_impl::exec(
        &*interface,
        module,
        function,
        param,
        remaining_gas,
        env.get_gas_costs(),
    )?;
    if cfg!(not(feature = "gas_calibration")) {
        set_remaining_points(&env, ctx, resp.remaining_gas)?;
    }
    env.get_interface().finish_call()?;
    Ok(resp)
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

    let remaining_gas = if cfg!(feature = "gas_calibration") {
        u64::MAX
    } else {
        get_remaining_points(&env, ctx)?
    };

    let module = interface.get_module(&bytecode, remaining_gas)?;
    let resp = crate::execution_impl::exec(
        &*interface,
        module,
        function,
        param,
        remaining_gas,
        env.get_gas_costs(),
    )?;
    if cfg!(not(feature = "gas_calibration")) {
        set_remaining_points(&env, ctx, resp.remaining_gas)?;
    }
    Ok(resp)
}

pub(crate) fn function_exists(
    ctx: &mut FunctionEnvMut<ASEnv>,
    address: &str,
    function: &str,
) -> ABIResult<bool> {
    let env = ctx.data().clone();
    let interface = env.get_interface();
    let bytecode = interface.raw_get_bytecode_for(address)?;

    let remaining_gas = if cfg!(feature = "gas_calibration") {
        u64::MAX
    } else {
        get_remaining_points(&env, ctx)?
    };

    match interface.get_module(&bytecode, remaining_gas)? {
        RuntimeModule::ASModule(module) => Ok(module
            .binary_module
            .exports()
            .functions()
            .any(|export| export.name() == function)),
    }
}

/// Create a smart contract with the given `bytecode`
pub(crate) fn create_sc(ctx: &mut FunctionEnvMut<ASEnv>, bytecode: &[u8]) -> ABIResult<String> {
    let env = ctx.data();
    Ok(env.get_interface().create_module(bytecode)?)
}
