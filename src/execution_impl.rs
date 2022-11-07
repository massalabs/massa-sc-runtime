use crate::execution::{create_instance, get_module, MassaModule};
use crate::settings;
use crate::types::{Interface, Response};
use anyhow::{bail, Result};
use wasmer::Instance;
use wasmer_middlewares::metering::{self, MeteringPoints};

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
    instance: Option<Instance>,
    mut module: impl MassaModule,
    function: &str,
    param: &str,
) -> Result<Response> {
    let instance = match instance {
        Some(instance) => instance,
        None => create_instance(limit, &module)?,
    };
    module.init_with_instance(&instance)?;

    match module.execution(&instance, function, param) {
        Ok(response) => Ok(response),
        Err(err) => {
            if cfg!(feature = "gas_calibration") {
                bail!(err)
            } else {
                // Because the last needed more than the remaining points, we should have an error.
                match metering::get_remaining_points(&instance) {
                    MeteringPoints::Remaining(..) => bail!(err),
                    MeteringPoints::Exhausted => bail!("Not enough gas, limit reached at: {function}"),
                }
            }
        }
    }
}

pub (crate) fn exec2(instance: Instance, mut module: impl MassaModule,
                     function: &str, param: &str) -> Result<(Response, Instance)> {

    module.init_with_instance(&instance)?;
    match module.execution(&instance, function, param) {
        Ok(response) => Ok((response, instance)),
        Err(err) => {
            bail!(err)
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
    let instance = create_instance(limit, &module)?;
    if instance.exports.contains(settings::MAIN) {
        let exec_res = exec(limit, Some(instance.clone()), module, settings::MAIN, "");
        Ok(exec_res?.remaining_gas)
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
    param: &str,
    interface: &dyn Interface,
) -> Result<u64> {
    let module = get_module(interface, bytecode)?;
    Ok(exec(limit, None, module, function, param)?.remaining_gas)
}
