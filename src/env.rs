//! Extends the env of wasmer-as

use crate::abi_impl::{abi_bail, get_memory, ABIResult};
use crate::types::Interface;
use anyhow::Result;
use as_ffi_bindings::{Read, StringPtr};
use std::sync::{Arc, Mutex};
use wasmer::{HostEnvInitError, Instance, WasmerEnv};
use wasmer_middlewares::metering::{self, set_remaining_points, MeteringPoints};

lazy_static! {
    pub static ref ENV: Arc<Mutex<Option<Env>>> = Arc::new(Mutex::new(None));
}

/// Error that append when a smartcontract try to call massa-std outside the
/// main function. Wasmer hasn't the time to call `fn init_with_instance`
pub const EXEC_INSTANCE_ERR: &str = "Unable to create execution instance. Make
sure that you are not trying to call massa-std outside the main function";

#[derive(Clone)]
pub struct Env {
    pub wasm_env: as_ffi_bindings::Env,
    pub interface: Box<dyn Interface>,
    pub instance: Option<Instance>,
}

impl Env {
    pub fn new(interface: &dyn Interface) -> Env {
        Env {
            wasm_env: Default::default(),
            interface: interface.clone_box(),
            instance: None,
        }
    }
}

impl WasmerEnv for Env {
    fn init_with_instance(&mut self, instance: &Instance) -> Result<(), HostEnvInitError> {
        self.wasm_env.init_with_instance(instance)?;
        self.instance = Some(instance.clone());
        Ok(())
    }
}

pub fn get_remaining_points_for_env(env: &Env) -> ABIResult<u64> {
    let instance = match env.instance.clone() {
        Some(instance) => instance,
        None => abi_bail!(EXEC_INSTANCE_ERR),
    };
    Ok(match metering::get_remaining_points(&instance) {
        MeteringPoints::Remaining(gas) => gas,
        MeteringPoints::Exhausted => 0,
    })
}

pub fn get_remaining_points_for_instance(instance: &Instance) -> u64 {
    match metering::get_remaining_points(instance) {
        MeteringPoints::Remaining(gas) => gas,
        MeteringPoints::Exhausted => 0,
    }
}

pub fn sub_remaining_gas(env: &Env, gas: u64) -> ABIResult<()> {
    let instance = match env.instance.clone() {
        Some(instance) => instance,
        None => abi_bail!(EXEC_INSTANCE_ERR),
    };
    let remaining_gas = get_remaining_points_for_env(env)?;
    if let Some(remaining_gas) = remaining_gas.checked_sub(gas) {
        set_remaining_points(&instance, remaining_gas);
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
    message: StringPtr,
    filename: StringPtr,
    line: i32,
    col: i32,
) -> ABIResult<()> {
    let env = ENV.lock().expect("Couldn't acquire lock on env.");
    let env = match env.as_ref() {
        Some(env) => env,
        None => abi_bail!("Uninitialized host env."),
    };
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
