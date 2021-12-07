use crate::types::Interface;
use anyhow::Result;
/// Extends the env of wasmer-as
///
///
use wasmer::{HostEnvInitError, Instance, WasmerEnv};
use wasmer_as::{Read, StringPtr};
use wasmer_middlewares::metering::{self, MeteringPoints};

#[derive(Clone)]
pub struct Env {
    pub wasm_env: wasmer_as::Env,
    pub remaining_points: u64,
    pub interface: Interface,
}

impl Env {
    pub fn new(interface: &Interface) -> Env {
        Env {
            wasm_env: Default::default(),
            remaining_points: Default::default(),
            interface: interface.clone(),
        }
    }
}

impl WasmerEnv for Env {
    fn init_with_instance(&mut self, instance: &Instance) -> Result<(), HostEnvInitError> {
        self.wasm_env.init_with_instance(instance)?;
        self.remaining_points = get_remaining_points(instance);
        Ok(())
    }
}

pub fn get_remaining_points(instance: &Instance) -> u64 {
    match metering::get_remaining_points(&instance) {
        MeteringPoints::Remaining(point) => point,
        MeteringPoints::Exhausted => 0,
    }
}

// if get_string throws an exception abort for some reason is being called
pub fn abort(env: &Env, message: StringPtr, filename: StringPtr, line: i32, col: i32) {
    let memory = env.wasm_env.memory.get_ref().expect("initialized memory");
    let message = message.read(memory).unwrap();
    let filename = filename.read(memory).unwrap();
    eprintln!("Error: {} at {}:{} col: {}", message, filename, line, col);
}
