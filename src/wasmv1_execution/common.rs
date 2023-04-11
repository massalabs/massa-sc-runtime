//! Execution functions used by the ABIs.
//!
//! IMPORTANT: these were designed for, and should not be called outside of ABIs.

/*

use wasmer::FunctionEnvMut;
use wasmer_middlewares::metering::get_remaining_points;

use super::{env::WasmV1ABIEnv, WasmV1Error};
use crate::Response;


/// Calls an exported function in a WASM module at a given address
pub(crate) fn call_module(
    ctx: &mut FunctionEnvMut<WasmV1ABIEnv>,
    address: &str,
    function: &str,
    param: &[u8],
    raw_coins: i64,
) -> Result<Response, WasmV1Error> {
    let raw_coins: u64 = match raw_coins.try_into() {
        Ok(v) => v,
        Err(_) => abi_bail!("negative amount of coins in Call"),
    };
    let env = get_env(ctx)?;
    let bytecode = env.get_interface().init_call(address, raw_coins)?;
    let interface = env.get_interface();

    let remaining_gas = if cfg!(feature = "gas_calibration") {
        u64::MAX
    } else {
        get_remaining_points(&env, ctx)
    };

    let module = interface.get_module(&bytecode, remaining_gas)?;
    let resp = crate::execution::run_function(
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
    ctx: &mut FunctionEnvMut<WasmV1ABIEnv>,
    bytecode: &[u8],
    function: &str,
    param: &[u8],
) -> Result<Response, WasmV1Error> {
    let env = get_env(ctx)?;
    let interface = env.get_interface();

    let remaining_gas = if cfg!(feature = "gas_calibration") {
        u64::MAX
    } else {
        get_remaining_points(&env, ctx)?
    };

    let module = interface.get_module(bytecode, remaining_gas)?;
    let resp = crate::execution::run_function(
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

/// Create a smart contract with the given `bytecode`
pub(crate) fn create_sc(
    ctx: &mut FunctionEnvMut<WasmV1ABIEnv>,
    bytecode: &[u8],
) -> ABIResult<String> {
    let env = ctx.data();
    Ok(env.get_interface().create_module(bytecode)?)
}

/// Check the exports of a compiled module to see if it contains the given function
pub(crate) fn function_exists(
    ctx: &mut FunctionEnvMut<WasmV1ABIEnv>,
    address: &str,
    function: &str,
) -> ABIResult<bool> {
    let env = get_env(ctx)?;
    let interface = env.get_interface();
    let bytecode = interface.raw_get_bytecode_for(address)?;

    let remaining_gas = if cfg!(feature = "gas_calibration") {
        u64::MAX
    } else {
        get_remaining_points(&env, ctx)?
    };

    Ok(interface
        .get_module(&bytecode, remaining_gas)?
        .function_exists(function))
}
*/
