//! Extends the env of wasmer-as

use crate::{
    env::{get_memory, sub_remaining_gas},
    execution::{abi_bail, ABIResult},
    settings,
    types::Interface,
};
use anyhow::Result;
use as_ffi_bindings::{Read, StringPtr};
use std::collections::HashMap;
use wasmer::{Extern, Global, HostEnvInitError, Instance, WasmerEnv};

use super::MassaEnv;

#[derive(Clone)]
pub struct ASEnv {
    wasm_env: as_ffi_bindings::Env,
    interface: Box<dyn Interface>,
    remaining_points: Option<Global>,
    exhausted_points: Option<Global>,
    param_size_map: HashMap<String, Option<Global>>,
}

impl MassaEnv<as_ffi_bindings::Env> for ASEnv {
    fn new(interface: &dyn Interface) -> Self {
        Self {
            wasm_env: Default::default(),
            interface: interface.clone_box(),
            remaining_points: None,
            exhausted_points: None,
            param_size_map: Default::default(),
        }
    }
    fn get_exhausted_points(&self) -> Option<&Global> {
        self.exhausted_points.as_ref()
    }
    fn get_remaining_points(&self) -> Option<&Global> {
        self.remaining_points.as_ref()
    }
    fn get_gc_param(&self, name: &str) -> Option<&Global> {
        self.param_size_map.get(name)?.as_ref()
    }
    fn get_interface(&self) -> Box<dyn Interface> {
        self.interface.clone()
    }
    fn get_wasm_env(&self) -> &as_ffi_bindings::Env {
        &self.wasm_env
    }
}

impl WasmerEnv for ASEnv {
    fn init_with_instance(&mut self, instance: &Instance) -> Result<(), HostEnvInitError> {
        self.wasm_env.init_with_instance(instance)?;

        if cfg!(feature = "gas_calibration") {
            // Find exports (like wgc_ps_massa.assembly_script_print) and get weak ref
            for (ex_name, extern_idx) in instance
                .exports
                .iter()
                .filter(|(ex_name, _)| ex_name.starts_with("wgc_ps"))
            {
                match extern_idx {
                    Extern::Global(_global) => {
                        let global_ref = instance
                            .exports
                            .get_with_generics_weak(ex_name)
                            .map_err(HostEnvInitError::from)?;

                        self.param_size_map
                            .insert((*ex_name).clone(), Some(global_ref));
                    }
                    _ => {
                        println!("Unhandled exports: {}", ex_name)
                    }
                }
            }
        } else {
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
        }

        Ok(())
    }
}

/// Called by the instance when an error popped. It print the filename where the error
/// had pop up, an error message and more stacktrace information as line and column
///
/// This function is automatically exported by AssemblyScript on build and allow assemblyscript
/// to log what happened when a smartcontract crashed inside the instance.
///
/// Because AssemblyScript require this to be imported:
/// - To create an instance, this function has to be in the ImportObject in the "env" namespace.
/// - We can take advantage of the behaviours printing the assemblyscript error
pub fn assembly_script_abort(
    env: &ASEnv,
    message: StringPtr,
    filename: StringPtr,
    line: i32,
    col: i32,
) -> ABIResult<()> {
    let memory = get_memory!(env);
    let message = message.read(memory);
    let filename = filename.read(memory);
    if message.is_err() || filename.is_err() {
        abi_bail!("aborting failed to load message or filename")
    }
    abi_bail!(format!(
        "error: {} at {}:{} col: {}",
        message.unwrap(),
        filename.unwrap(),
        line,
        col
    ));
}

/// Assembly script builtin export `seed` function
pub fn assembly_script_seed(env: &ASEnv) -> ABIResult<f64> {
    if cfg!(not(feature = "gas_calibration")) {
        sub_remaining_gas(env, settings::metering_unsafe_random())?;
    }
    match env.interface.unsafe_random_f64() {
        Ok(ret) => Ok(ret),
        _ => abi_bail!("failed to get random from interface"),
    }
}

/// Assembly script builtin `Date.now()`.
///
/// Note for developpers: It seems that AS as updated the output of that function
/// for the newest versions. Probably the signature will be soon () -> i64
/// instead of () -> f64.
pub fn assembly_script_date(env: &ASEnv) -> ABIResult<f64> {
    if cfg!(not(feature = "gas_calibration")) {
        sub_remaining_gas(env, settings::metering_get_time())?;
    }
    let utime = match env.interface.get_time() {
        Ok(time) => time,
        _ => abi_bail!("failed to get time from interface"),
    };
    let ret = utime as f64;
    Ok(ret)
}
