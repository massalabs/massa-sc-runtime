use crate::as_execution::{init_engine, init_store, ASContextModule, ASModule, RuntimeModule};
use crate::settings;
use crate::types::{Interface, Response};
use crate::GasCosts;

use anyhow::{bail, Result};
use tracing::warn;
use wasmer::Instance;
use wasmer_middlewares::metering::{self, MeteringPoints};

#[cfg(feature = "gas_calibration")]
use crate::middlewares::gas_calibration::{get_gas_calibration_result, GasCalibrationResult};

/// Select and launch the adequate execution function
pub(crate) fn exec(
    interface: &dyn Interface,
    module: RuntimeModule,
    function: &str,
    param: &[u8],
    limit: u64,
    prev_init_cost: Option<u64>,
    gas_costs: GasCosts,
) -> Result<Response> {
    let response = match module {
        RuntimeModule::ASModule((module, _engine)) => {
            exec_as_module(
                interface,
                module,
                function,
                param,
                limit,
                prev_init_cost,
                gas_costs,
            )?
            .0
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
/// The return of the function executed as string and the remaining gas for the rest of the execution.
pub(crate) fn exec_as_module(
    interface: &dyn Interface,
    as_module: ASModule,
    function: &str,
    param: &[u8],
    limit: u64,
    prev_init_cost: Option<u64>,
    gas_costs: GasCosts,
) -> Result<(Response, Instance)> {
    if let Some(_prev_init_cost) = prev_init_cost && limit < _prev_init_cost {
        bail!("critical: limit is lower than prev_init_cost");
    }

    warn!("(RUNTIME) limit = {}", limit);
    let engine = init_engine(limit, gas_costs.clone())?;
    let mut store = init_store(&engine)?;
    let mut context_module = ASContextModule::new(interface, as_module.binary_module, gas_costs);
    let (instance, init_rem_points) = context_module.create_vm_instance_and_init_env(&mut store)?;
    let init_cost = limit.saturating_sub(init_rem_points);
    warn!(
        "(RUNTIME) init_cost = {} | prev_init_cost = {:?}",
        init_cost, prev_init_cost
    );

    if let Some(_prev_init_cost) = prev_init_cost && init_cost != _prev_init_cost {
        bail!("critical: prev_init_cost and init_cost should have the same value");
    }

    metering::set_remaining_points(&mut store, &instance, limit.saturating_sub(init_cost));

    match context_module.execution(&mut store, &instance, function, param) {
        Ok(response) => Ok((response, instance)),
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
    module: RuntimeModule,
    limit: u64,
    gas_costs: GasCosts,
) -> Result<Response> {
    warn!("(RUNTIME) run_main");
    Ok(exec(
        interface,
        module,
        settings::MAIN,
        b"",
        limit,
        None,
        gas_costs,
    )?)
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
    module: RuntimeModule,
    function: &str,
    param: &[u8],
    limit: u64,
    prev_init_cost: Option<u64>,
    gas_costs: GasCosts,
) -> Result<Response> {
    Ok(exec(
        interface,
        module,
        function,
        param,
        limit,
        prev_init_cost,
        gas_costs,
    )?)
}

/// Same as run_main but return a GasCalibrationResult
#[cfg(feature = "gas_calibration")]
pub fn run_main_gc(
    bytecode: &[u8],
    limit: u64,
    interface: &dyn Interface,
    param: &[u8],
    gas_costs: GasCosts,
) -> Result<GasCalibrationResult> {
    // IMPORTANT TODO: consult how we'd like update this
    let mut module = get_module(interface, bytecode, gas_costs)?;
    let (instance, store) = create_instance(limit, &mut module)?;
    if instance.exports.contains(settings::MAIN) {
        let (_resp, instance, mut store) = exec(
            u64::MAX,
            Some((instance.clone(), store)),
            module,
            settings::MAIN,
            param,
        )?;
        Ok(get_gas_calibration_result(&instance, &mut store))
    } else {
        bail!("No main");
    }
}
