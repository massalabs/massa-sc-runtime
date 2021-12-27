//! Extends the env of wasmer-as

use crate::abi_impl::abi_bail;
use crate::types::Interface;
use anyhow::Result;
use as_ffi_bindings::{Read, StringPtr};
use wasmer::{HostEnvInitError, Instance, WasmerEnv};
use wasmer_middlewares::metering::{self, set_remaining_points, MeteringPoints};

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

pub fn get_remaining_points_for_env(env: &Env) -> u64 {
    let instance = &env.instance.clone().unwrap();
    match metering::get_remaining_points(instance) {
        MeteringPoints::Remaining(point) => point,
        MeteringPoints::Exhausted => 0,
    }
}

pub fn get_remaining_points_for_instance(instance: &Instance) -> u64 {
    match metering::get_remaining_points(instance) {
        MeteringPoints::Remaining(point) => point,
        MeteringPoints::Exhausted => 0,
    }
}

pub fn sub_remaining_point(env: &Env, points: u64) -> anyhow::Result<()> {
    let instance = &env.instance.clone().unwrap();
    let remaining_points = get_remaining_points_for_env(env);
    set_remaining_points(instance, remaining_points.saturating_sub(points));
    Ok(())
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
) {
    let memory = env.wasm_env.memory.get_ref().expect("initialized memory");
    let message = message.read(memory);
    let filename = filename.read(memory);
    if message.is_err() || filename.is_err() {
        abi_bail!("Aborting failed to load massage or filename")
    }
    eprintln!(
        "Error: {} at {}:{} col: {}",
        message.unwrap(),
        filename.unwrap(),
        line,
        col
    );
}
