mod abi;
mod common;
mod context;
pub(crate) mod env;
mod error;

use crate::{
    error::{exec_bail, VMResult},
    execution::Compiler,
    middlewares::{
        condom::CondomMiddleware,
        dumper::Dumper,
        gas_calibration::{get_gas_calibration_result, GasCalibration, GasCalibrationResult},
    },
    settings::max_number_of_pages,
    tunable_memory::LimitingTunables,
    CondomLimits, GasCosts, Interface, Response, VMError,
};
use std::sync::Arc;
use wasmer::{
    sys::{BaseTunables, EngineBuilder, Features},
    wasmparser::Operator,
    CompilerConfig, Cranelift, Engine, Module, NativeEngineExt, Pages, Store, Target,
};
use wasmer_compiler_singlepass::Singlepass;
use wasmer_middlewares::{
    metering::{self, MeteringPoints},
    Metering,
};

pub(crate) use context::*;
pub(crate) use error::*;

/// An executable runtime module compiled from an AssemblyScript SC
#[derive(Clone)]
pub struct ASModule {
    pub(crate) binary_module: Module,
    pub(crate) initial_limit: u64,
    pub compiler: Compiler,
    // Compilation engine can not be dropped
    pub(crate) _engine: Engine,
}

impl ASModule {
    pub(crate) fn new(
        bytecode: &[u8],
        limit: u64,
        gas_costs: GasCosts,
        compiler: Compiler,
        condom_limits: CondomLimits,
    ) -> VMResult<Self> {
        let engine = match compiler {
            Compiler::CL => init_cl_engine(limit, gas_costs, condom_limits),
            Compiler::SP => init_sp_engine(limit, gas_costs, condom_limits),
        };
        Ok(Self {
            binary_module: Module::new(&engine, bytecode)
                .map_err(|e| VMError::InstanceError(e.to_string()))?,
            initial_limit: limit,
            compiler,
            _engine: engine,
        })
    }

    pub fn serialize(&self) -> VMResult<Vec<u8>> {
        match self.compiler {
            Compiler::CL => Ok(self
                .binary_module
                .serialize()
                .map_err(|e| VMError::InstanceError(e.to_string()))?
                .to_vec()),
            Compiler::SP => {
                panic!("cannot serialize a module compiled with Singlepass")
            }
        }
    }

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
        // serialized by ourselves before
        let module = unsafe {
            Module::deserialize(&store, ser_module)
                .map_err(|e| VMError::InstanceError(e.to_string()))?
        };
        Ok(ASModule {
            binary_module: module,
            initial_limit: limit,
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

    // Add condom middleware
    compiler_config.push_middleware(Arc::new(CondomMiddleware::new(condom_limits)));

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

    // Add condom middleware
    compiler_config.push_middleware(Arc::new(CondomMiddleware::new(condom_limits)));

    if cfg!(feature = "gas_calibration") {
        // Add gas calibration middleware
        let gas_calibration = Arc::new(GasCalibration::new());
        compiler_config.push_middleware(gas_calibration);
    } else if cfg!(feature = "dumper") {
        // Add dumper middleware
        let dumper = Arc::new(Dumper::new());
        compiler_config.push_middleware(dumper);
    } else {
        // Add metering middleware
        let metering = Arc::new(Metering::new(limit, move |_: &Operator| -> u64 {
            gas_costs.operator_cost
        }));
        compiler_config.push_middleware(metering);
    }

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
/// * Output of the executed function, remaininng gas after execution and the
///   initialization cost
/// * Gas calibration result if it has been enabled
pub(crate) fn exec_as_module(
    interface: &dyn Interface,
    as_module: ASModule,
    function: &str,
    param: &[u8],
    limit: u64,
    gas_costs: GasCosts,
    condom_limits: CondomLimits,
) -> VMResult<(Response, Option<GasCalibrationResult>)> {
    let engine = match as_module.compiler {
        Compiler::CL => init_cl_engine(limit, gas_costs.clone(), condom_limits.clone()),
        Compiler::SP => init_sp_engine(limit, gas_costs.clone(), condom_limits.clone()),
    };
    let mut store = Store::new(engine);
    let mut context = ASContext::new(
        interface,
        as_module.binary_module,
        gas_costs,
        condom_limits.clone(),
    );

    // save the gas remaining before sub-execution: used by readonly execution
    interface.save_gas_remaining_before_subexecution(limit);

    let (instance, _fenv, init_rem_points) = context
        .create_vm_instance_and_init_env(&mut store)
        .map_err(|e| VMError::InstanceError(e.to_string()))?;
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
            response.init_gas_cost = init_cost;

            #[cfg(feature = "execution-trace")]
            {
                response.trace = _fenv.as_ref(&store).trace.clone();
            }

            Ok((response, gc_result))
        }
        Err(err) => {
            if cfg!(feature = "gas_calibration") {
                exec_bail!(err, init_cost)
            } else {
                // some error need to be handled carefully (depth error)
                // hence we match on the error type to handle specific cases
                match err {
                    VMError::DepthError(e) => Err(VMError::DepthError(e)),
                    _ => {
                        // Because the last needed more than the remaining points, we
                        // should have an error.
                        match metering::get_remaining_points(&mut store, &instance) {
                            MeteringPoints::Remaining(..) => {
                                exec_bail!(err, init_cost)
                            }
                            MeteringPoints::Exhausted => {
                                exec_bail!(
                                    format!("Not enough gas, limit reached at: {function}"),
                                    init_cost
                                )
                            }
                        }
                    }
                }
            }
        }
    }
}
