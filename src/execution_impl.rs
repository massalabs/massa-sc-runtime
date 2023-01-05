use crate::execution::{BinaryModule, ContextModule};
use crate::settings;
use crate::types::{Interface, Response};

use anyhow::{bail, Result};
use wasmer::Instance;
use wasmer_middlewares::metering::{self, MeteringPoints};

#[cfg(feature = "gas_calibration")]
use crate::middlewares::gas_calibration::{get_gas_calibration_result, GasCalibrationResult};

/// Internal execution function, used on smart contract called from node or
/// from another smart contract
/// Parameters:
/// * `limit`: Limit of gas that can be used.
/// * `instance`: Optional wasmer instance to be passed instead of creating it directly in the function.
/// * `module`: Bytecode that contains the function to be executed.
/// * `function`: Name of the function to call.
/// * `param`: Parameter to pass to the function.
/// * `interface`: Interface to call function in Massa from execution context.
///
/// Return:
/// The return of the function executed as string and the remaining gas for the rest of the execution.
pub(crate) fn exec(
    interface: &dyn Interface,
    binary_module: impl BinaryModule,
    function: &str,
    param: &[u8],
) -> Result<(Response, Instance)> {
    // IMPORTANT TODO: update doc for all the runtime modifications
    let mut context_module = ContextModule::new(interface, binary_module);
    let instance = context_module.create_vm_instance_and_init_env()?;

    match context_module.execution(&instance, function, param) {
        Ok(response) => Ok((response, instance)),
        Err(err) => {
            if cfg!(feature = "gas_calibration") {
                bail!(err)
            } else {
                // Because the last needed more than the remaining points, we should have an error.
                match metering::get_remaining_points(&instance) {
                    MeteringPoints::Remaining(..) => bail!(err),
                    MeteringPoints::Exhausted => {
                        bail!("Not enough gas, limit reached at: {function}")
                    }
                }
            }
        }
    }
}

/// Library Input, take a `module` wasm built with the massa environment,
/// must have a main function inside written in AssemblyScript:
///
/// ```js
/// import { print } from "massa-sc-std";
///
/// export function main(_args: string): i32 {
///     print("hello world");
///     return 0;
/// }
/// ```
/// Return:
/// the remaining gas.
pub fn run_main(binary_module: impl BinaryModule, interface: &dyn Interface) -> Result<u64> {
    // REVIEW NOTE: there is actually no need to check if MAIN exists here since execution will
    // produce an error if it doesnt, which is actually what you would expect and not the other
    // way around imho
    Ok(exec(interface, binary_module, settings::MAIN, b"")?
        .0
        .remaining_gas)
}

/// Library Input, take a `module` wasm built with the massa environment,
/// run a function of that module with the given parameter:
///
/// ```js
/// import { print } from "massa-sc-std";
///
/// export function hello_world(_args: string): i32 {
///     print("hello world");
///     return 0;
/// }
/// ```
pub fn run_function(
    binary_module: impl BinaryModule,
    function: &str,
    param: &[u8],
    interface: &dyn Interface,
) -> Result<u64> {
    Ok(exec(interface, binary_module, function, param)?
        .0
        .remaining_gas)
}

/// Same as run_main but return a GasCalibrationResult
#[cfg(feature = "gas_calibration")]
pub fn run_main_gc(
    bytecode: &[u8],
    limit: u64,
    interface: &dyn Interface,
) -> Result<GasCalibrationResult> {
    // IMPORTANT TODO: consult how we'd like to have this update and update it
    let module = get_module(interface, bytecode)?;
    let instance = create_instance(limit, &module)?;
    if instance.exports.contains(settings::MAIN) {
        let (_resp, instance) = exec(
            u64::MAX,
            Some(instance.clone()),
            module,
            settings::MAIN,
            b"",
        )?;
        Ok(get_gas_calibration_result(&instance))
    } else {
        bail!("No main");
    }
}
