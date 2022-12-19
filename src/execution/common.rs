use wasmer::WasmerEnv;

use crate::env::{get_remaining_points, set_remaining_points, MassaEnv};
use crate::Response;
use displaydoc::Display;
use thiserror::Error;

use super::get_module;

pub(crate) type ABIResult<T, E = ABIError> = core::result::Result<T, E>;

#[derive(Display, Error, Debug)]
pub enum ABIError {
    /// Runtime error: {0}
    Error(#[from] anyhow::Error),
    /// Runtime wasmer error: {0}
    WasmerError(#[from] wasmer::RuntimeError),
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
pub(crate) fn call_module<T: WasmerEnv>(
    env: &impl MassaEnv<T>,
    address: &str,
    function: &str,
    param: &[u8],
    raw_coins: i64,
) -> ABIResult<Response> {
    let raw_coins: u64 = match raw_coins.try_into() {
        Ok(v) => v,
        Err(_) => abi_bail!("negative amount of coins in Call"),
    };
    let bytecode = env.get_interface().init_call(address, raw_coins)?;
    let module = get_module(&*env.get_interface(), &bytecode)?;

    let remaining_gas = if cfg!(feature = "gas_calibration") {
        Ok(u64::MAX)
    } else {
        get_remaining_points(env)
    };

    let resp = crate::execution_impl::exec(remaining_gas?, None, module, function, param)?;
    if cfg!(not(feature = "gas_calibration")) {
        set_remaining_points(env, resp.0.remaining_gas)?;
    }
    env.get_interface().finish_call()?;
    Ok(resp.0)
}

/// Alternative to `call_module` to execute bytecode in a local context
pub(crate) fn local_call<T: WasmerEnv>(
    env: &impl MassaEnv<T>,
    bytecode: &[u8],
    function: &str,
    param: &[u8],
) -> ABIResult<Response> {
    let module = get_module(&*env.get_interface(), bytecode)?;

    let remaining_gas = if cfg!(feature = "gas_calibration") {
        Ok(u64::MAX)
    } else {
        get_remaining_points(env)
    };

    let resp = crate::execution_impl::exec(remaining_gas?, None, module, function, param)?;
    if cfg!(not(feature = "gas_calibration")) {
        set_remaining_points(env, resp.0.remaining_gas)?;
    }
    Ok(resp.0)
}

/// Create a smart contract with the given `bytecode`
pub(crate) fn create_sc<T: WasmerEnv>(
    env: &impl MassaEnv<T>,
    bytecode: &[u8],
) -> ABIResult<String> {
    Ok(env.get_interface().create_module(bytecode)?)
}
