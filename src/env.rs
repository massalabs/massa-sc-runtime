use crate::types::Interface;
use anyhow::Result;
/// Extends the env of wasmer-as
///
///
use wasmer::{HostEnvInitError, Instance, WasmerEnv};
use wasmer_as::{Read, StringPtr};
use wasmer_middlewares::metering::{self, set_remaining_points, MeteringPoints};

#[derive(Clone)]
pub struct Env {
    pub wasm_env: wasmer_as::Env,
    pub interface: Interface,
    pub instance: Option<Instance>,
}

impl Env {
    pub fn new(interface: &Interface) -> Env {
        Env {
            wasm_env: Default::default(),
            interface: interface.clone(),
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

pub fn get_remaining_points(env: &Env) -> u64 {
    let instance = &env.instance.clone().unwrap();
    match metering::get_remaining_points(instance) {
        MeteringPoints::Remaining(point) => point,
        MeteringPoints::Exhausted => 0,
    }
}

pub fn get_remaining_points_instance(instance: &Instance) -> u64 {
    match metering::get_remaining_points(instance) {
        MeteringPoints::Remaining(point) => point,
        MeteringPoints::Exhausted => 0,
    }
}

pub fn sub_remaining_point(env: &Env, points: u64) -> anyhow::Result<()> {
    let instance = &env.instance.clone().unwrap();
    set_remaining_points(instance, get_remaining_points(env) - points);
    Ok(())
}

// if get_string throws an exception abort for some reason is being called
pub fn abort(env: &Env, message: StringPtr, filename: StringPtr, line: i32, col: i32) {
    let memory = env.wasm_env.memory.get_ref().expect("initialized memory");
    let message = message.read(memory).unwrap();
    let filename = filename.read(memory).unwrap();
    eprintln!("Error: {} at {}:{} col: {}", message, filename, line, col);
}
