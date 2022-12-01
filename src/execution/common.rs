use crate::env::{get_remaining_points, set_remaining_points, MassaEnv};
use crate::Response;

use super::get_module;

pub(crate) type ABIResult<T, E = wasmer::RuntimeError> = core::result::Result<T, E>;
macro_rules! abi_bail {
    ($err:expr) => {
        return Err(wasmer::RuntimeError::new($err.to_string()))
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
    let bytecode = &match env.get_interface().init_call(address, raw_coins) {
        Ok(bytecode) => bytecode,
        Err(err) => abi_bail!(err),
    };
    let module = match get_module(&*env.get_interface(), bytecode) {
        Ok(module) => module,
        Err(err) => abi_bail!(err),
    };

    let remaining_gas = if cfg!(feature = "gas_calibration") {
        Ok(u64::MAX)
    } else {
        get_remaining_points(env)
    };

    match crate::execution_impl::exec(remaining_gas?, None, module, function, param) {
        Ok(resp) => {
            if cfg!(not(feature = "gas_calibration")) {
                if let Err(err) = set_remaining_points(env, resp.0.remaining_gas) {
                    abi_bail!(err);
                }
            }
            match env.get_interface().finish_call() {
                Ok(_) => Ok(resp.0),
                Err(err) => abi_bail!(err),
            }
        }
        Err(err) => abi_bail!(err),
    }
}

/// Create a smart contract with the given `bytecode`
pub(crate) fn create_sc<T: WasmerEnv>(
    env: &impl MassaEnv<T>,
    bytecode: &[u8],
) -> ABIResult<String> {
    match env.get_interface().create_module(bytecode) {
        Ok(address) => Ok(address),
        Err(err) => abi_bail!(err),
    }
}
