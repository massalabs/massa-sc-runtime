mod abi;
mod env;
mod error;
mod ffi;

use self::env::{ABIEnv, ExecutionEnv};
use crate::error::VMResult;
use crate::execution::Compiler;
use crate::middlewares::condom::CondomMiddleware;
use crate::middlewares::gas_calibration::{
    get_gas_calibration_result, GasCalibration, GasCalibrationResult,
};
use crate::settings::max_number_of_pages;
use crate::tunable_memory::LimitingTunables;
use crate::{CondomLimits, GasCosts, Interface, Response, VMError};
use abi::*;
pub(crate) use error::*;
use parking_lot::Mutex;
use std::sync::Arc;
use wasmer::NativeEngineExt;
use wasmer::{sys::Features, CompilerConfig, Cranelift, Engine, Module, Store};
use wasmer::{
    sys::{BaseTunables, EngineBuilder},
    wasmparser::Operator,
    Pages, Target,
};
use wasmer_compiler_singlepass::Singlepass;
use wasmer_middlewares::Metering;

/// An executable runtime module compiled from an AssemblyScript SC
#[derive(Clone)]
pub struct WasmV1Module {
    pub(crate) binary_module: Module,
    pub(crate) gas_limit_at_compilation: u64,
    pub compiler: Compiler,
    // Compilation engine can not be dropped
    pub(crate) _engine: Engine,
}

impl WasmV1Module {
    pub(crate) fn compile(
        bytecode: &[u8],
        limit: u64,
        gas_costs: GasCosts,
        compiler: Compiler,
        condom_limits: CondomLimits,
    ) -> Result<Self, WasmV1Error> {
        let engine = match compiler {
            Compiler::CL => init_cl_engine(limit, gas_costs, condom_limits),
            Compiler::SP => init_sp_engine(limit, gas_costs, condom_limits),
        };
        let binary_module = match Module::new(&engine, bytecode) {
            Ok(module) => module,
            Err(e) => {
                return Err(WasmV1Error::InstanciationError(format!(
                    "Could not compile bytecode: {}",
                    e
                )))
            }
        };
        Ok(Self {
            binary_module,
            gas_limit_at_compilation: limit,
            compiler,
            _engine: engine,
        })
    }

    /// Serialize a module
    pub fn serialize(&self) -> Vec<u8> {
        match self.compiler {
            Compiler::CL => self
                .binary_module
                .serialize()
                .expect("Could not serialize module")
                .to_vec(),
            Compiler::SP => {
                panic!("cannot serialize a module compiled with Singlepass")
            }
        }
    }

    /// Deserialize a module
    pub fn deserialize(
        ser_module: &[u8],
        limit: u64,
        gas_costs: GasCosts,
        condom_limits: CondomLimits,
    ) -> VMResult<Self> {
        // Deserialization is only meant for Cranelift modules
        let engine = init_cl_engine(limit, gas_costs, condom_limits);
        let store = Store::new(engine.clone());
        // Unsafe because code injection is possible
        // That's not an issue because we only deserialize modules we have
        // serialized by ourselves before. This also means that we
        // expect the module to be valid and the deserialization to
        // succeed.
        let binary_module = unsafe { Module::deserialize(&store, ser_module) }
            .expect("Could not deserialize module");
        Ok(WasmV1Module {
            binary_module,
            gas_limit_at_compilation: limit,
            compiler: Compiler::CL,
            _engine: engine,
        })
    }

    /// Check the exports of a compiled module to see if it contains the given
    /// function
    pub(crate) fn function_exists(&self, function: &str) -> bool {
        self.binary_module
            .exports()
            .functions()
            .any(|export| export.name() == function)
    }
}

// Compiler feature.
// Turn off all sources of non-determinism.
//
// References:
// * https://github.com/webassembly/bulk-memory-operations
// * https://github.com/WebAssembly/design/blob/390bab47efdb76b600371bcef1ec0ea374aa8c43/Nondeterminism.md
// * https://github.com/WebAssembly/proposals
//
// TLDR: Turn off every feature except for `bulk_memory`.
const FEATURES: Features = Features {
    threads: false,         // non-deterministic
    reference_types: false, // could be enabled but we have no need for it atm
    simd: false,            // non-deterministic
    bulk_memory: true,      // enables the use of buffers in AS
    multi_value: false,     // could be enabled but we have no need for it atm
    tail_call: false,       // experimental
    module_linking: false,  // experimental
    multi_memory: false,    // experimental
    memory64: false,        // experimental
    exceptions: false,      // experimental
    relaxed_simd: false,    // experimental
    extended_const: false,  // experimental
};

pub(crate) fn init_sp_engine(
    limit: u64,
    gas_costs: GasCosts,
    condom_limits: CondomLimits,
) -> Engine {
    // Singlepass is used to compile arbitrary bytecode.
    //
    // Reference:
    // * https://docs.rs/wasmer-compiler-singlepass/latest/wasmer_compiler_singlepass/
    let mut compiler_config = Singlepass::new();

    // Canonicalize NaN
    compiler_config.canonicalize_nans(true);
    add_middleware(&mut compiler_config, limit, gas_costs, condom_limits);

    let base = BaseTunables::for_target(&Target::default());
    let tunables = LimitingTunables::new(base, Pages(max_number_of_pages()));

    let mut engine = Engine::from(
        EngineBuilder::new(compiler_config)
            .set_features(Some(FEATURES))
            .engine(),
    );
    engine.set_tunables(tunables);
    engine
}

pub(crate) fn init_cl_engine(
    limit: u64,
    gas_costs: GasCosts,
    condom_limits: CondomLimits,
) -> Engine {
    // Cranelift is used to compile bytecode that will be cached.
    //
    // Reference:
    // * https://docs.rs/wasmer-compiler-cranelift/latest/wasmer_compiler_cranelift/
    let mut compiler_config = Cranelift::new();

    // Canonicalize NaN
    compiler_config.canonicalize_nans(true);
    add_middleware(&mut compiler_config, limit, gas_costs, condom_limits);

    let base = BaseTunables::for_target(&Target::default());
    let tunables = LimitingTunables::new(base, Pages(max_number_of_pages()));

    let mut engine = Engine::from(
        EngineBuilder::new(compiler_config)
            .set_features(Some(FEATURES))
            .engine(),
    );
    engine.set_tunables(tunables);
    engine
}

fn add_middleware<T>(
    compiler_config: &mut T,
    limit: u64,
    gas_costs: GasCosts,
    condom_limits: CondomLimits,
) where
    T: CompilerConfig,
{
    // Add condom middleware
    let condom_middleware = Arc::new(CondomMiddleware::new(condom_limits));
    compiler_config.push_middleware(condom_middleware);

    if cfg!(feature = "gas_calibration") {
        // Add gas calibration middleware
        let gas_calibration = Arc::new(GasCalibration::new());
        compiler_config.push_middleware(gas_calibration);
    } else {
        // Add metering middleware
        let metering = Arc::new(Metering::new(limit, move |_: &Operator| -> u64 {
            gas_costs.operator_cost
        }));
        compiler_config.push_middleware(metering);
    }
}

pub(crate) fn exec_wasmv1_module(
    interface: &dyn Interface,
    module: WasmV1Module,
    function: &str,
    param: &[u8],
    gas_limit: u64,
    gas_costs: GasCosts,
    condom_limits: CondomLimits,
) -> VMResult<(Response, Option<GasCalibrationResult>)> {
    // Init store
    let engine = match module.compiler {
        Compiler::CL => init_cl_engine(gas_limit, gas_costs.clone(), condom_limits.clone()),
        Compiler::SP => init_sp_engine(gas_limit, gas_costs.clone(), condom_limits.clone()),
    };
    let mut store = Store::new(engine);

    // Create the ABI imports and pass them an empty environment for now
    let shared_abi_env: ABIEnv = Arc::new(Mutex::new(None));

    let interface_version = interface.get_interface_version().unwrap_or(0);

    let import_object = register_abis(&mut store, shared_abi_env.clone(), interface_version);

    // save the gas remaining before subexecution: used by readonly execution
    interface.save_gas_remaining_before_subexecution(gas_limit);

    // Create an instance of the execution environment.
    let execution_env = ExecutionEnv::create_instance(
        &mut store,
        &module,
        interface,
        gas_costs,
        &import_object,
        condom_limits,
    )
    .map_err(|err| {
        VMError::InstanceError(format!(
            "Failed to create instance of execution environment: {}",
            err
        ))
    })?;

    // Get gas cost of instance creation
    let init_gas_cost = execution_env.get_init_gas_cost();

    // Set gas limit of function execution by subtracting the gas cost of
    // instance creation
    let available_gas = match gas_limit.checked_sub(init_gas_cost) {
        Some(remaining_gas) => remaining_gas,
        None => {
            return Err(VMError::ExecutionError {
                error: "Available gas does not cover instance creation".to_string(),
                init_gas_cost,
            })
        }
    };
    execution_env.set_remaining_gas(&mut store, available_gas);

    // Get function to execute. Must follow the following prototype: param_addr:
    // i32 -> return_addr: i32
    let wasm_func =
        execution_env
            .get_func(&store, function)
            .map_err(|err| VMError::ExecutionError {
                error: format!(
                    "Could not find guest function {} for call: {}",
                    function, err
                ),
                init_gas_cost,
            })?;

    // Allocate and write function argument to guest memory
    let param_offset = execution_env
        .create_buffer(&mut store, param)
        .map_err(|err| VMError::ExecutionError {
            error: format!(
                "Could not write argument for guest call {}: {}",
                function, err
            ),
            init_gas_cost,
        })?;

    // Now that we have an instance, we can make the execution environment
    // available to the ABIs. We avoided setting it before instance creation
    // to prevent the implicit `_start` call from accessing the env and
    // causing non-determinism in init gas usage.
    shared_abi_env.lock().replace(execution_env);

    // Call func
    let returned_offset =
        wasm_func
            .call(&mut store, param_offset)
            .map_err(|err| VMError::ExecutionError {
                error: format!("Error while calling guest function {}: {}", function, err),
                init_gas_cost,
            })?;

    // Take back the execution environment
    let execution_env = shared_abi_env
        .lock()
        .take()
        .expect("Execution environment unavailable after execution");

    // Read returned value from guest memory and deallocate it
    let ret = execution_env
        .take_buffer(&mut store, returned_offset)
        .map_err(|err| VMError::ExecutionError {
            error: format!(
                "Could not read return value from guest call {}: {}",
                function, err
            ),
            init_gas_cost,
        })?;

    // Get remaining gas
    let remaining_gas = execution_env.get_remaining_gas(&mut store);

    let gc_result = if cfg!(feature = "gas_calibration") {
        Some(get_gas_calibration_result(
            &execution_env.instance,
            &mut store,
        ))
    } else {
        None
    };

    // Return response
    Ok((
        Response {
            ret,
            remaining_gas,
            init_gas_cost,
            #[cfg(feature = "execution-trace")]
            trace: execution_env.trace.clone(),
        },
        gc_result,
    ))
}
