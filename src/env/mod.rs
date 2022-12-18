mod as_env;

use crate::{
    execution::{abi_bail, ABIResult},
    Interface, settings::GAS_COSTS,
};
pub(crate) use as_env::*;
use wasmer::{Global, WasmerEnv};

macro_rules! get_memory {
    ($env:ident) => {
        match $env.get_wasm_env().memory.get_ref() {
            Some(mem) => mem,
            _ => abi_bail!("uninitialized memory"),
        }
    };
}
pub(crate) use get_memory;

pub(crate) trait MassaEnv<T: WasmerEnv>: WasmerEnv {
    fn new(interface: &dyn Interface) -> Self;
    fn get_exhausted_points(&self) -> Option<&Global>;
    fn get_remaining_points(&self) -> Option<&Global>;
    fn get_gc_param(&self, name: &str) -> Option<&Global>;
    fn get_interface(&self) -> Box<dyn Interface>;
    fn get_wasm_env(&self) -> &T;
}

/// Get remaining metering points
/// Should be equivalent to
/// https://github.com/wasmerio/wasmer/blob/8f2e49d52823cb7704d93683ce798aa84b6928c8/lib/middlewares/src/metering.rs#L293
pub(crate) fn get_remaining_points<T: WasmerEnv>(env: &impl MassaEnv<T>) -> ABIResult<u64> {
    if cfg!(feature = "gas_calibration") {
        Ok(u64::MAX)
    } else {
        match env.get_exhausted_points().as_ref() {
            Some(exhausted_points) => match exhausted_points.get().try_into() {
                Ok::<i32, _>(exhausted) if exhausted > 0 => return Ok(0),
                Ok::<i32, _>(_) => (),
                Err(_) => abi_bail!("exhausted_points has wrong type"),
            },
            None => abi_bail!("Lost reference to exhausted_points"),
        };
        match env.get_remaining_points().as_ref() {
            Some(remaining_points) => match remaining_points.get().try_into() {
                Ok::<u64, _>(remaining) => Ok(remaining),
                Err(_) => abi_bail!("remaining_points has wrong type"),
            },
            None => abi_bail!("Lost reference to remaining_points"),
        }
    }
}

/// Set remaining metering points
/// Should be equivalent to
/// https://github.com/wasmerio/wasmer/blob/8f2e49d52823cb7704d93683ce798aa84b6928c8/lib/middlewares/src/metering.rs#L343
pub(crate) fn set_remaining_points<T: WasmerEnv>(
    env: &impl MassaEnv<T>,
    points: u64,
) -> ABIResult<()> {
    if cfg!(not(feature = "gas_calibration")) {
        match env.get_remaining_points().as_ref() {
            Some(remaining_points) => {
                if remaining_points.set(points.into()).is_err() {
                    abi_bail!("Can't set remaining_points");
                }
            }
            None => abi_bail!("Lost reference to remaining_points"),
        };
        match env.get_exhausted_points().as_ref() {
            Some(exhausted_points) => {
                if exhausted_points.set(0i32.into()).is_err() {
                    abi_bail!("Can't set exhausted_points")
                }
            }
            None => abi_bail!("Lost reference to exhausted_points"),
        };
    }
    Ok(())
}

pub(crate) fn sub_remaining_gas<T: WasmerEnv>(env: &impl MassaEnv<T>, gas: u64) -> ABIResult<()> {
    if cfg!(feature = "gas_calibration") {
        return Ok(());
    }
    let remaining_gas = get_remaining_points(env)?;
    if let Some(remaining_gas) = remaining_gas.checked_sub(gas) {
        set_remaining_points(env, remaining_gas)?;
    } else {
        abi_bail!("Remaining gas reach zero")
    }
    Ok(())
}

pub(crate) fn sub_remaining_gas_abi<T: WasmerEnv>(
    env: &impl MassaEnv<T>,
    abi_name: &str,
) -> ABIResult<()> {
    sub_remaining_gas(
        env,
        *GAS_COSTS
            .get(abi_name)
            .ok_or(wasmer::RuntimeError::new(format!(
                "Failed to get gas for {} ABI",
                abi_name
            )))?,
    )
}
