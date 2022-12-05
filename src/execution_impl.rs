use crate::execution::{create_instance, get_module, MassaModule};
use crate::settings;
use crate::types::{Interface, Response};

use anyhow::{bail, Result};
use wasmer::{Instance, Store};
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
    limit: u64,
    instance_and_store: Option<(Instance, Store)>,
    mut module: impl MassaModule,
    function: &str,
    param: &[u8],
) -> Result<(Response, Instance)> {
    let (instance, mut store) = match instance_and_store {
        Some((instance, store)) => (instance, store),
        None => create_instance(limit, &module)?,
    };
    module.init_with_instance(&instance, &store)?;

    match module.execution(&instance, &mut store, function, param) {
        Ok(response) => {
            Ok((response, instance))
        },
        Err(err) => {
            if cfg!(feature = "gas_calibration") {
                bail!(err)
            } else {
                // Because the last needed more than the remaining points, we should have an error.
                match metering::get_remaining_points(&mut store, &instance) {
                    MeteringPoints::Remaining(..) => {
                        bail!(err)
                    },
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
pub fn run_main(bytecode: &[u8], limit: u64, interface: &dyn Interface) -> Result<u64> {
    let module = get_module(interface, bytecode)?;
    let (instance, store) = create_instance(limit, &module)?;
    if instance.exports.contains(settings::MAIN) {
        Ok(exec(limit, Some((instance, store)), module, settings::MAIN, b"")?
            .0
            .remaining_gas)
    } else {
        Ok(limit)
    }
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
    bytecode: &[u8],
    limit: u64,
    function: &str,
    param: &[u8],
    interface: &dyn Interface,
) -> Result<u64> {
    let module = get_module(interface, bytecode)?;
    Ok(exec(limit, None, module, function, param)?.0.remaining_gas)
}

/// Same as run_main but return a GasCalibrationResult
#[cfg(feature = "gas_calibration")]
pub fn run_main_gc(
    bytecode: &[u8],
    limit: u64,
    interface: &dyn Interface,
) -> Result<GasCalibrationResult> {
    let module = get_module(interface, bytecode)?;
    let (instance, store) = create_instance(limit, &module)?;
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
