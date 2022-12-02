//! Extends the env of wasmer-as

use crate::{
    env::{get_memory, sub_remaining_gas},
    execution::{abi_bail, ABIResult},
    settings,
    types::Interface,
};
use anyhow::Result;
use as_ffi_bindings::{Read, StringPtr};
use wasmer::{FunctionEnvMut, Global, Instance};

use super::MassaEnv;

#[derive(Clone)]
pub struct ASEnv {
    wasm_env: as_ffi_bindings::Env,
    interface: Box<dyn Interface>,
    remaining_points: Option<Global>,
    exhausted_points: Option<Global>,
}

impl MassaEnv<as_ffi_bindings::Env> for ASEnv {
    fn new(interface: &dyn Interface) -> Self {
        Self {
            wasm_env: Default::default(),
            interface: interface.clone_box(),
            remaining_points: None,
            exhausted_points: None,
        }
    }
    fn get_exhausted_points(&self) -> Option<&Global> {
        self.exhausted_points.as_ref()
    }
    fn get_remaining_points(&self) -> Option<&Global> {
        self.remaining_points.as_ref()
    }
    fn get_interface(&self) -> Box<dyn Interface> {
        self.interface.clone()
    }
    fn get_wasm_env(&self) -> &as_ffi_bindings::Env {
        &self.wasm_env
    }
    fn get_wasm_env_as_mut(&mut self) -> &mut as_ffi_bindings::Env {
        &mut self.wasm_env
    }
}

/*
impl WasmerEnv for ASEnv {
    fn init_with_instance(&mut self, instance: &Instance) -> Result<(), HostEnvInitError> {
        self.wasm_env.init_with_instance(instance)?;

        if cfg!(not(feature = "gas_calibration")) {
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
*/

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
    ctx: FunctionEnvMut<ASEnv>,
    message: StringPtr,
    filename: StringPtr,
    line: i32,
    col: i32,
) -> ABIResult<()> {

    let memory = ctx.data().get_wasm_env().memory.as_ref().expect("mem??").clone();
    let message_ = message
        .read(&memory, &ctx)
        .map_err(|e| wasmer::RuntimeError::new(e.to_string()));
    let filename_ = filename
        .read(&memory, &ctx)
        .map_err(|e| wasmer::RuntimeError::new(e.to_string()));

    if message_.is_err() || filename_.is_err() {
        abi_bail!("aborting failed to load message or filename")
    }
    abi_bail!(format!(
        "error: {} at {}:{} col: {}",
        message_.unwrap(),
        filename_.unwrap(),
        line,
        col
    ));
}

/// Assembly script builtin export `seed` function
pub fn assembly_script_seed(mut ctx: FunctionEnvMut<ASEnv>) -> ABIResult<f64> {

    let env = ctx.data().clone();
    if cfg!(not(feature = "gas_calibration")) {
        sub_remaining_gas(&env, &mut ctx, settings::metering_unsafe_random())?;
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
pub fn assembly_script_date(mut ctx: FunctionEnvMut<ASEnv>) -> ABIResult<f64> {

    let env = ctx.data().clone();
    if cfg!(not(feature = "gas_calibration")) {
        sub_remaining_gas(&env, &mut ctx, settings::metering_get_time())?;
    }

    let utime = match env.interface.get_time() {
        Ok(time) => time,
        _ => abi_bail!("failed to get time from interface"),
    };
    let ret = utime as f64;
    Ok(ret)
}
