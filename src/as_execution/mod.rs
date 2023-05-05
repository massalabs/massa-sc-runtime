mod abi;
mod common;
mod context;
mod env;
mod error;

use crate::error::{exec_bail, VMResult};
use crate::execution::Compiler;
use crate::middlewares::gas_calibration::{get_gas_calibration_result, GasCalibrationResult};
use crate::middlewares::{dumper::Dumper, gas_calibration::GasCalibration};
use crate::settings::max_number_of_pages;
use crate::tunable_memory::LimitingTunables;
use crate::{GasCosts, Interface, Response};
use anyhow::Result;
use std::sync::Arc;
use wasmer::{wasmparser::Operator, BaseTunables, Engine, EngineBuilder, Pages, Target};
use wasmer::{CompilerConfig, Cranelift, Features, Module, Store};
use wasmer_compiler_singlepass::Singlepass;
use wasmer_middlewares::metering::MeteringPoints;
use wasmer_middlewares::{metering, Metering};

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
    ) -> Result<Self> {
        let engine = match compiler {
            Compiler::CL => init_cl_engine(limit, gas_costs),
            Compiler::SP => init_sp_engine(limit, gas_costs),
        };
        Ok(Self {
            binary_module: Module::new(&engine, bytecode)?,
            initial_limit: limit,
            compiler,
            _engine: engine,
        })
    }

    pub fn serialize(&self) -> Result<Vec<u8>> {
        match self.compiler {
            Compiler::CL => Ok(self.binary_module.serialize()?.to_vec()),
            Compiler::SP => panic!("cannot serialize a module compiled with Singlepass"),
        }
    }

    pub fn deserialize(ser_module: &[u8], limit: u64, gas_costs: GasCosts) -> Result<Self> {
        // Deserialization is only meant for Cranelift modules
        let mut engine = init_cl_engine(limit, gas_costs);
        let store = init_store(&mut engine)?;
        // Unsafe because code injection is possible
        // That's not an issue because we only deserialize modules we have serialized by ourselves before
        let module = unsafe { Module::deserialize(&store, ser_module)? };
        Ok(ASModule {
            binary_module: module,
            initial_limit: limit,
            compiler: Compiler::CL,
            _engine: engine,
        })
    }

    /// Check the exports of a compiled module to see if it contains the given function
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

pub(crate) fn init_sp_engine(limit: u64, gas_costs: GasCosts) -> Engine {
    // Singlepass is used to compile arbitrary bytecode.
    //
    // Reference:
    // * https://docs.rs/wasmer-compiler-singlepass/latest/wasmer_compiler_singlepass/
    let mut compiler_config = Singlepass::new();

    // Canonicalize NaN
    compiler_config.canonicalize_nans(true);

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

    Engine::from(
        EngineBuilder::new(compiler_config)
            .set_features(Some(FEATURES))
            .engine(),
    )
}

pub(crate) fn init_cl_engine(limit: u64, gas_costs: GasCosts) -> Engine {
    // Cranelift is used to compile bytecode that will be cached.
    //
    // Reference:
    // * https://docs.rs/wasmer-compiler-cranelift/latest/wasmer_compiler_cranelift/
    let mut compiler_config = Cranelift::new();

    // Canonicalize NaN
    compiler_config.canonicalize_nans(true);

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

    Engine::from(
        EngineBuilder::new(compiler_config)
            .set_features(Some(FEATURES))
            .engine(),
    )
}

pub(crate) fn init_store(engine: &mut Engine) -> Result<Store> {
    let base = BaseTunables::for_target(&Target::default());
    let tunables = LimitingTunables::new(base, Pages(max_number_of_pages()));
    engine.set_tunables(tunables);
    let store = Store::new(engine.clone());
    Ok(store)
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
) -> VMResult<(Response, Option<GasCalibrationResult>)> {
    let mut engine = match as_module.compiler {
        Compiler::CL => init_cl_engine(limit, gas_costs.clone()),
        Compiler::SP => init_sp_engine(limit, gas_costs.clone()),
    };
    let mut store = init_store(&mut engine)?;
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
            response.init_gas_cost = init_cost;
            Ok((response, gc_result))
        }
        Err(err) => {
            if cfg!(feature = "gas_calibration") {
                exec_bail!(err, init_cost)
            } else {
                // Because the last needed more than the remaining points, we should have an error.
                match metering::get_remaining_points(&mut store, &instance) {
                    MeteringPoints::Remaining(..) => exec_bail!(err, init_cost),
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
