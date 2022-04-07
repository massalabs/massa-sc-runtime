//! Extends the env of wasmer-as

use crate::abi_impl::{abi_bail, get_memory, ABIResult};
use crate::types::Interface;
use anyhow::Result;
use as_ffi_bindings::{Read, StringPtr};
use wasmer::{Global, HostEnvInitError, Instance, WasmerEnv};

#[derive(Clone)]
pub struct Env {
    pub wasm_env: as_ffi_bindings::Env,
    pub interface: Box<dyn Interface>,
    pub remaining_points: Option<Global>,
    pub exhausted_points: Option<Global>,
}

impl Env {
    pub fn new(interface: &dyn Interface) -> Env {
        Env {
            wasm_env: Default::default(),
            interface: interface.clone_box(),
            remaining_points: None,
            exhausted_points: None,
        }
    }
}

impl WasmerEnv for Env {
    fn init_with_instance(&mut self, instance: &Instance) -> Result<(), HostEnvInitError> {
        self.wasm_env.init_with_instance(instance)?;
        self.remaining_points = Some(
            instance
                .exports
                .get_with_generics_weak("wasmer_metering_remaining_points")
                .map_err(HostEnvInitError::from)?,
        );
        self.exhausted_points = Some(
            instance
                .exports
                .get_with_generics_weak("wasmer_metering_points_exhausted")
                .map_err(HostEnvInitError::from)?,
        );
        Ok(())
    }
}

/// Get remaining metering points
/// Should be equivalent to
/// https://github.com/wasmerio/wasmer/blob/8f2e49d52823cb7704d93683ce798aa84b6928c8/lib/middlewares/src/metering.rs#L293
pub fn get_remaining_points(env: &Env) -> ABIResult<u64> {
    match env.exhausted_points.as_ref() {
        Some(exhausted_points) => match exhausted_points.get().try_into() {
            Ok::<i32, _>(exhausted) if exhausted > 0 => return Ok(0),
            Ok::<i32, _>(_) => (),
            Err(_) => abi_bail!("exhausted_points has wrong type"),
        },
        None => abi_bail!("Lost reference to exhausted_points"),
    };
    match env.remaining_points.as_ref() {
        Some(remaining_points) => match remaining_points.get().try_into() {
            Ok::<u64, _>(remaining) => Ok(remaining),
            Err(_) => abi_bail!("remaining_points has wrong type"),
        },
        None => abi_bail!("Lost reference to remaining_points"),
    }
}

/// Set remaining metering points
/// Should be equivalent to 
/// https://github.com/wasmerio/wasmer/blob/8f2e49d52823cb7704d93683ce798aa84b6928c8/lib/middlewares/src/metering.rs#L343
fn set_remaining_points(env: &Env, points: u64) -> ABIResult<()> {
    match env.remaining_points.as_ref() {
        Some(remaining_points) => {
            if remaining_points.set(points.into()).is_err() {
                abi_bail!("Can't set remaining_points");
            }
        }
        None => abi_bail!("Lost reference to remaining_points"),
    };
    match env.exhausted_points.as_ref() {
        Some(exhausted_points) => {
            if exhausted_points.set(0i32.into()).is_err() {
                abi_bail!("Can't set exhausted_points")
            }
        }
        None => abi_bail!("Lost reference to exhausted_points"),
    };
    Ok(())
}

pub fn sub_remaining_gas(env: &Env, gas: u64) -> ABIResult<()> {
    let remaining_gas = get_remaining_points(env)?;
    if let Some(remaining_gas) = remaining_gas.checked_sub(gas) {
        set_remaining_points(env, remaining_gas)?;
    } else {
        abi_bail!("Remaining gas reach zero")
    }
    Ok(())
}

/// Try to substract remaining gas computing the gas with a*b and ceiling
/// the result.
pub fn sub_remaining_gas_with_mult(env: &Env, a: usize, b: usize) -> ABIResult<()> {
    match a.checked_mul(b) {
        Some(gas) => sub_remaining_gas(env, gas as u64),
        None => abi_bail!(format!("Multiplication overflow {} {}", a, b)),
    }
}

/// Called by the instance when an error popped. It print the filename where the error
/// had pop up, an error message and more stacktrace information as line and column
///
/// This function is automatically exported by AssemblyScript on build and allow assemblyscript
/// to log what appened when a smartcontract crashed inside the instance.
///
/// Because AssemblyScript require this to be imported:
/// - To create an instance, this function has to be in the ImportObject in the "env" namespace.
/// - We can take advantage of the behaviours printing the assemblyscript error
pub fn assembly_script_abort(
    env: &Env,
    message: StringPtr,
    filename: StringPtr,
    line: i32,
    col: i32,
) -> ABIResult<()> {
    let memory = get_memory!(env);
    let message = message.read(memory);
    let filename = filename.read(memory);
    if message.is_err() || filename.is_err() {
        abi_bail!("Aborting failed to load message or filename")
    }
    eprintln!(
        "Error: {} at {}:{} col: {}",
        message.unwrap(),
        filename.unwrap(),
        line,
        col
    );
    Ok(())
}
