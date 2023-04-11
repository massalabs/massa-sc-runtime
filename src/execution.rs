use crate::as_execution::{exec_as_module, ASModule};
use crate::error::VMResult;
use crate::middlewares::gas_calibration::GasCalibrationResult;
use crate::settings;
use crate::types::{Interface, Response};
use crate::wasmv1_execution::{exec_wasmv1_module, WasmV1Module};
use crate::GasCosts;
use anyhow::{anyhow, Result};

/// Enum listing the available compilers
#[derive(Clone)]
pub enum Compiler {
    CL,
    SP,
}

#[derive(Clone)]
pub enum RuntimeModule {
    ASModule(ASModule),
    WasmV1Module(WasmV1Module),
}

impl RuntimeModule {
    /// Dispatch module creation corresponding to the first bytecode byte
    ///
    /// * (1) TODO: target AssemblyScript (remove ident)
    /// * (2) TODO: target X
    /// * (_) target AssemblyScript
    pub fn new(
        bytecode: &[u8],
        limit: u64,
        gas_costs: GasCosts,
        compiler: Compiler,
    ) -> Result<Self> {
        match bytecode.first() {
            Some(0) => Ok(Self::ASModule(ASModule::new(
                bytecode, limit, gas_costs, compiler,
            )?)),
            Some(1) => {
                let res = WasmV1Module::compile(&bytecode[1..], limit, gas_costs, compiler)
                    .map_err(|err| anyhow!("Failed to compile WasmV1 module: {}", err))?;
                Ok(Self::WasmV1Module(res))
            }
            Some(v) => Err(anyhow!("Unsupported bytecode type: {}", v)),
            None => Err(anyhow!("Empty bytecode")),
        }
    }

    /// Used compiler for the current module
    pub fn compiler(&self) -> Compiler {
        match self {
            RuntimeModule::ASModule(module) => module.compiler.clone(),
            RuntimeModule::WasmV1Module(module) => module.compiler.clone(),
        }
    }

    /// Serialize a RuntimeModule
    ///
    /// TODO: set a module identifier for other types of sub modules.
    /// Distinction between runtime module ident and sub module ident must be clear.
    /// If the serialization process becomes too complex use NOM.
    pub fn serialize(&self) -> Result<Vec<u8>> {
        let ser = match self {
            RuntimeModule::ASModule(module) => module.serialize()?,
            RuntimeModule::WasmV1Module(module) => module.serialize(),
        };
        Ok(ser)
    }

    /// Deserialize a RuntimeModule
    ///
    /// NOTE: only deserialize from ASModule for now
    /// TODO: make a distinction based on the runtime module identifier byte (see `serialize` description)
    pub fn deserialize(ser_module: &[u8], limit: u64, gas_costs: GasCosts) -> Result<Self> {
        let deser = RuntimeModule::ASModule(ASModule::deserialize(ser_module, limit, gas_costs)?);
        Ok(deser)
    }

    /// Check the exports of a compiled module to see if it contains the given function
    pub(crate) fn function_exists(&self, function: &str) -> bool {
        match self {
            RuntimeModule::ASModule(module) => module.function_exists(function),
            RuntimeModule::WasmV1Module(module) => module.function_exists(function),
        }
    }
}

/// Select and launch the adequate execution function
pub(crate) fn exec(
    interface: &dyn Interface,
    rt_module: RuntimeModule,
    function: &str,
    param: &[u8],
    limit: u64,
    gas_costs: GasCosts,
) -> VMResult<(Response, Option<GasCalibrationResult>)> {
    let response = match rt_module {
        RuntimeModule::ASModule(module) => {
            exec_as_module(interface, module, function, param, limit, gas_costs)?
        }
        RuntimeModule::WasmV1Module(module) => {
            let res = exec_wasmv1_module(interface, module, function, param, limit, gas_costs)
                .map_err(|err| anyhow!("Failed to execute WasmV1 module: {}", err.to_string()))?;
            (res, None) // TODO add gas calibration
        }
    };
    Ok(response)
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
) -> VMResult<Response> {
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
) -> VMResult<Response> {
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
) -> VMResult<GasCalibrationResult> {
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
