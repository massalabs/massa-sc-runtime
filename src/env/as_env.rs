//! Extends the env of wasmer-as

use crate::{
    as_execution::{abi_bail, ABIResult},
    env::sub_remaining_gas_abi,
    types::Interface,
    GasCosts,
};
use as_ffi_bindings::{Read, StringPtr};
use function_name::named;
use std::collections::HashMap;
use std::ops::Add;
use wasmer::{FunctionEnvMut, Global};

use super::MassaEnv;

#[derive(Clone)]
pub struct ASEnv {
    wasm_env: as_ffi_bindings::Env,
    interface: Box<dyn Interface>,
    pub remaining_points: Option<Global>,
    pub exhausted_points: Option<Global>,
    param_size_map: HashMap<String, Option<Global>>,
    gas_costs: GasCosts,
}

impl MassaEnv<as_ffi_bindings::Env> for ASEnv {
    fn new(interface: &dyn Interface, gas_costs: GasCosts) -> Self {
        Self {
            wasm_env: Default::default(),
            gas_costs,
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
    fn get_gas_costs(&self) -> GasCosts {
        self.gas_costs.clone()
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
    let memory = ctx
        .data()
        .get_wasm_env()
        .memory
        .as_ref()
        .expect("Failed to get memory on env")
        .clone();
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
#[named]
pub fn assembly_script_seed(mut ctx: FunctionEnvMut<ASEnv>) -> ABIResult<f64> {
    let env = ctx.data().clone();
    if cfg!(not(feature = "gas_calibration")) {
        sub_remaining_gas_abi(&env, &mut ctx, function_name!())?;
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
/// instead of () -> f64. This change is in AS 0.22 if we upgrade the version in our SCs we need to update this function.
#[named]
pub fn assembly_script_date_now(mut ctx: FunctionEnvMut<ASEnv>) -> ABIResult<f64> {
    let env = ctx.data().clone();
    if cfg!(not(feature = "gas_calibration")) {
        sub_remaining_gas_abi(&env, &mut ctx, function_name!())?;
    }
    let utime = match env.interface.get_time() {
        Ok(time) => time,
        _ => abi_bail!("failed to get time from interface"),
    };
    let ret = utime as f64;
    Ok(ret)
}

/// Assembly script builtin `console.log()`.
#[named]
pub fn assembly_script_console_log(
    mut ctx: FunctionEnvMut<ASEnv>,
    message: StringPtr,
) -> ABIResult<()> {
    let env = ctx.data().clone();
    if cfg!(not(feature = "gas_calibration")) {
        sub_remaining_gas_abi(&env, &mut ctx, function_name!())?;
    }
    assembly_script_console(ctx, message, "LOG")
}

/// Assembly script builtin `console.info()`.
#[named]
pub fn assembly_script_console_info(
    mut ctx: FunctionEnvMut<ASEnv>,
    message: StringPtr,
) -> ABIResult<()> {
    let env = ctx.data().clone();
    if cfg!(not(feature = "gas_calibration")) {
        sub_remaining_gas_abi(&env, &mut ctx, function_name!())?;
    }
    assembly_script_console(ctx, message, "INFO")
}

/// Assembly script builtin `console.warn()`.
#[named]
pub fn assembly_script_console_warn(
    mut ctx: FunctionEnvMut<ASEnv>,
    message: StringPtr,
) -> ABIResult<()> {
    let env = ctx.data().clone();
    if cfg!(not(feature = "gas_calibration")) {
        sub_remaining_gas_abi(&env, &mut ctx, function_name!())?;
    }
    assembly_script_console(ctx, message, "WARN")
}

/// Assembly script builtin `console.debug()`.
#[named]
pub fn assembly_script_console_debug(
    mut ctx: FunctionEnvMut<ASEnv>,
    message: StringPtr,
) -> ABIResult<()> {
    let env = ctx.data().clone();
    if cfg!(not(feature = "gas_calibration")) {
        sub_remaining_gas_abi(&env, &mut ctx, function_name!())?;
    }
    assembly_script_console(ctx, message, "DEBUG")
}

/// Assembly script builtin `console.error()`.
#[named]
pub fn assembly_script_console_error(
    mut ctx: FunctionEnvMut<ASEnv>,
    message: StringPtr,
) -> ABIResult<()> {
    let env = ctx.data().clone();
    if cfg!(not(feature = "gas_calibration")) {
        sub_remaining_gas_abi(&env, &mut ctx, function_name!())?;
    }
    assembly_script_console(ctx, message, "ERROR")
}

/// Assembly script console functions
fn assembly_script_console(
    ctx: FunctionEnvMut<ASEnv>,
    message: StringPtr,
    prefix: &str,
) -> ABIResult<()> {
    let env = ctx.data().clone();
    let memory = ctx
        .data()
        .get_wasm_env()
        .memory
        .as_ref()
        .expect("Failed to get memory on env")
        .clone();
    let message = prefix
        .to_string()
        .add(" | ")
        .add(&message.read(&memory, &ctx)?);
    env.get_interface().generate_event(message)?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
/// Assembly script builtin `trace()`.
#[named]
pub fn assembly_script_trace(
    mut ctx: FunctionEnvMut<ASEnv>,
    message: StringPtr,
    n: i32,
    a0: f64,
    a1: f64,
    a2: f64,
    a3: f64,
    a4: f64,
) -> ABIResult<()> {
    let env = ctx.data().clone();
    if cfg!(not(feature = "gas_calibration")) {
        sub_remaining_gas_abi(&env, &mut ctx, function_name!())?;
    }

    let memory = ctx
        .data()
        .get_wasm_env()
        .memory
        .as_ref()
        .expect("Failed to get memory on env")
        .clone();

    let message = message.read(&memory, &ctx)?;

    let message_for_event = match n {
        1 => format!("msg: {}, a0: {}", message, a0),
        2 => format!("msg: {}, a0: {}, a1: {}", message, a0, a1),
        3 => format!("msg: {}, a0: {}, a1: {}, a2: {}", message, a0, a1, a2),
        4 => format!(
            "msg: {}, a0: {}, a1: {}, a2: {}, a3: {}",
            message, a0, a1, a2, a3
        ),
        5 => format!(
            "msg: {}, a0: {}, a1: {}, a2: {}, a3: {}, a4: {}",
            message, a0, a1, a2, a3, a4
        ),
        _ => message, // Should we warn here or return an error?
    };

    env.get_interface().generate_event(message_for_event)?;
    Ok(())
}

/// Assembly script builtin `process.exit()`.
pub fn assembly_script_process_exit(_ctx: FunctionEnvMut<ASEnv>, exit_code: i32) -> ABIResult<()> {
    abi_bail!(format!("exit with code: {}", exit_code));
}
