use crate::as_execution::{
    init_cl_engine, init_sp_engine, init_store, ASContext, ASModule, RuntimeModule,
};
use crate::middlewares::gas_calibration::{get_gas_calibration_result, GasCalibrationResult};
use crate::settings;
use crate::types::{Interface, Response};
use crate::GasCosts;

use anyhow::{bail, Result};
use wasmer_middlewares::metering::{self, MeteringPoints};

/// Select and launch the adequate execution function
pub(crate) fn exec(
    interface: &dyn Interface,
    rt_module: RuntimeModule,
    function: &str,
    param: &[u8],
    limit: u64,
    gas_costs: GasCosts,
) -> Result<(Response, Option<GasCalibrationResult>)> {
    let response = match rt_module {
        RuntimeModule::ASModule(module) => {
            exec_as_module(interface, module, function, param, limit, gas_costs)?
        }
    };
    Ok(response)
}

/// Internal execution function, used on smart contract called from node or
/// from another smart contract
/// Parameters:
/// * `interface`: Interface to call function in Massa from execution context
/// * `as_module`: Pre compiled AS module that will be instantiated and executed
/// * `function`: Name of the function to call
/// * `param`: Parameter passed to the function
/// * `cache`: Cache of pre compiled modules
/// * `gas_costs`: Cost in gas of every VM operation
///
/// Return:
/// * Output of the executed function, remaininng gas after execution and the initialization cost
/// * Gas calibration result if it has been enabled
pub(crate) fn exec_as_module(
    interface: &dyn Interface,
    as_module: ASModule,
    function: &str,
    param: &[u8],
    limit: u64,
    gas_costs: GasCosts,
) -> Result<(Response, Option<GasCalibrationResult>)> {
    let engine = if as_module.cache_compatible {
        init_cl_engine(limit, gas_costs.clone())
    } else {
        init_sp_engine(limit, gas_costs.clone())
    };
    let mut store = init_store(&engine)?;
    let mut context = ASContext::new(interface, as_module.binary_module, gas_costs);
    let (instance, init_rem_points) = context.create_vm_instance_and_init_env(&mut store)?;
    let init_cost = as_module.initial_limit.saturating_sub(init_rem_points);

    if cfg!(not(feature = "gas_calibration")) {
        metering::set_remaining_points(&mut store, &instance, limit.saturating_sub(init_cost));
    }

    match context.execution(&mut store, &instance, function, param) {
        Ok(mut response) => {
            let gc_result = if cfg!(feature = "gas_calibration") {
                Some(get_gas_calibration_result(&instance, &mut store))
            } else {
                None
            };
            response.init_cost = init_cost;
            Ok((response, gc_result))
        }
        Err(err) => {
            if cfg!(feature = "gas_calibration") {
                bail!(err)
            } else {
                // Because the last needed more than the remaining points, we should have an error.
                match metering::get_remaining_points(&mut store, &instance) {
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
pub fn run_main(
    interface: &dyn Interface,
    rt_module: RuntimeModule,
    limit: u64,
    gas_costs: GasCosts,
) -> Result<Response> {
    Ok(exec(interface, rt_module, settings::MAIN, b"", limit, gas_costs)?.0)
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
    interface: &dyn Interface,
    rt_module: RuntimeModule,
    function: &str,
    param: &[u8],
    limit: u64,
    gas_costs: GasCosts,
) -> Result<Response> {
    Ok(exec(interface, rt_module, function, param, limit, gas_costs)?.0)
}

/// Same as run_main but return a GasCalibrationResult
#[cfg(feature = "gas_calibration")]
pub fn run_main_gc(
    interface: &dyn Interface,
    rt_module: RuntimeModule,
    param: &[u8],
    limit: u64,
    gas_costs: GasCosts,
) -> Result<GasCalibrationResult> {
    Ok(exec(
        interface,
        rt_module,
        settings::MAIN,
        param,
        limit,
        gas_costs,
    )?
    .1
    .unwrap())
}
