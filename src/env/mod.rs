mod as_env;

use wasmer::{AsStoreMut, Global, Store};
use crate::{
    execution::{abi_bail, ABIResult},
    Interface,
};
pub(crate) use as_env::*;

macro_rules! get_memory {
    ($env:ident) => {
        match $env.get_wasm_env().memory.get_ref() {
            Some(mem) => mem,
            _ => abi_bail!("uninitialized memory"),
        }
    };
}
pub(crate) use get_memory;

pub(crate) trait MassaEnv<T> {
    fn new(interface: &dyn Interface) -> Self;
    fn get_exhausted_points(&self) -> Option<&Global>;
    fn get_remaining_points(&self) -> Option<&Global>;
    fn get_interface(&self) -> Box<dyn Interface>;
    fn get_wasm_env(&self) -> &T;
    fn get_wasm_env_as_mut(&mut self) -> &mut T;
}

/// Get remaining metering points
/// Should be equivalent to
/// https://github.com/wasmerio/wasmer/blob/8f2e49d52823cb7704d93683ce798aa84b6928c8/lib/middlewares/src/metering.rs#L293
pub(crate) fn get_remaining_points<T>(env: &impl MassaEnv<T>, store: &mut impl AsStoreMut) -> ABIResult<u64> {
    if cfg!(feature = "gas_calibration") {
        return Ok(0);
    }

    match env.get_exhausted_points().as_ref() {
        Some(exhausted_points) => match exhausted_points.get(store).try_into() {
            Ok::<i32, _>(exhausted) if exhausted > 0 => return Ok(0),
            Ok::<i32, _>(_) => (),
            Err(_) => abi_bail!("exhausted_points has wrong type"),
        },
        None => abi_bail!("Lost reference to exhausted_points"),
    };
    match env.get_remaining_points().as_ref() {
        Some(remaining_points) => match remaining_points.get(store).try_into() {
            Ok::<u64, _>(remaining) => Ok(remaining),
            Err(_) => abi_bail!("remaining_points has wrong type"),
        },
        None => abi_bail!("Lost reference to remaining_points"),
    }
}

/// Set remaining metering points
/// Should be equivalent to
/// https://github.com/wasmerio/wasmer/blob/8f2e49d52823cb7704d93683ce798aa84b6928c8/lib/middlewares/src/metering.rs#L343
pub(crate) fn set_remaining_points<T>(
    env: &impl MassaEnv<T>,
    store: &mut impl AsStoreMut,
    points: u64,
) -> ABIResult<()> {
    match env.get_remaining_points().as_ref() {
        Some(remaining_points) => {
            if remaining_points.set(store,points.into()).is_err() {
                abi_bail!("Can't set remaining_points");
            }
        }
        None => abi_bail!("Lost reference to remaining_points"),
    };
    match env.get_exhausted_points().as_ref() {
        Some(exhausted_points) => {
            if exhausted_points.set(store, 0i32.into()).is_err() {
                abi_bail!("Can't set exhausted_points")
            }
        }
        None => abi_bail!("Lost reference to exhausted_points"),
    };
    Ok(())
}

pub(crate) fn sub_remaining_gas<T>(env: &impl MassaEnv<T>, store: &mut impl AsStoreMut, gas: u64) -> ABIResult<()> {
    if cfg!(feature = "gas_calibration") {
        return Ok(());
    }
    let remaining_gas = get_remaining_points(env, store)?;
    if let Some(remaining_gas) = remaining_gas.checked_sub(gas) {
        set_remaining_points(env, store,remaining_gas)?;
    } else {
        abi_bail!("Remaining gas reach zero")
    }
    Ok(())
}

/// Try to subtract remaining gas computing the gas with a*b and ceiling
/// the result.
pub(crate) fn sub_remaining_gas_with_mult<T>(
    env: &impl MassaEnv<T>,
    store: &mut Store,
    a: usize,
    b: usize,
) -> ABIResult<()> {
    match a.checked_mul(b) {
        Some(gas) => sub_remaining_gas(env, store, gas as u64),
        None => abi_bail!(format!("Multiplication overflow {a} {b}")),
    }
}
