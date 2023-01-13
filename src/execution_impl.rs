use std::sync::Arc;

use crate::execution::{init_engine, init_store, ContextModule};
use crate::types::{Interface, Response};
use crate::GasCosts;
use crate::{settings, ModuleCache};

use anyhow::{bail, Result};
use parking_lot::RwLock;
use wasmer::{Engine, Instance, Module};
use wasmer_middlewares::metering::{self, MeteringPoints};

#[cfg(feature = "gas_calibration")]
use crate::middlewares::gas_calibration::{get_gas_calibration_result, GasCalibrationResult};

/// Internal execution function, used on smart contract called from node or
/// from another smart contract
/// Parameters:
/// * `interface`: Interface to call function in Massa from execution context
/// * `engine`: Engine used for store creation and module compilation
/// * `binary_module`: Pre compiled module that will be instantiated and executed
/// * `function`: Name of the function to call
/// * `param`: Parameter passed to the function
/// * `cache`: Cache of pre compiled modules
/// * `gas_costs`: Cost in gas of every VM operation
///
/// Return:
/// The return of the function executed as string and the remaining gas for the rest of the execution.
pub(crate) fn exec(
    interface: &dyn Interface,
    engine: &Engine,
    binary_module: Module,
    function: &str,
    param: &[u8],
    cache: Arc<RwLock<ModuleCache>>,
    gas_costs: GasCosts,
) -> Result<(Response, Instance)> {
    let mut store = init_store(engine)?;
    let mut context_module = ContextModule::new(interface, binary_module, cache, gas_costs);
    let instance = context_module.create_vm_instance_and_init_env(&mut store)?;

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
    bytecode: &[u8],
    cache: Arc<RwLock<ModuleCache>>,
    limit: u64,
    gas_costs: GasCosts,
) -> Result<Response> {
    // NOTE: do not use cache in `run_main` as it is used for sc execution only
    // NOTE: match bytecode target ident and init a different engine accordingly here
    let engine = init_engine(limit, gas_costs.clone())?;
    let binary_module = Module::new(&engine, bytecode)?;
    Ok(exec(
        interface,
        &engine,
        binary_module,
        settings::MAIN,
        b"",
        cache,
        gas_costs,
    )?
    .0)
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
    bytecode: &[u8],
    function: &str,
    param: &[u8],
    cache: Arc<RwLock<ModuleCache>>,
    limit: u64,
    gas_costs: GasCosts,
) -> Result<Response> {
    // NOTE: match bytecode target ident and init a different engine accordingly here
    let engine = init_engine(limit, gas_costs.clone())?;
    let binary_module = cache.write().get_module(&engine, bytecode)?;
    Ok(exec(
        interface,
        &engine,
        binary_module,
        function,
        param,
        cache,
        gas_costs,
    )?
    .0)
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
