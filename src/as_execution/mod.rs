mod abi;
mod common;
mod context;
mod error;

use anyhow::{anyhow, Result};
use std::sync::Arc;
use wasmer::{wasmparser::Operator, BaseTunables, EngineBuilder, Pages, Target};
use wasmer::{CompilerConfig, Cranelift, Engine, Features, Module, Store};
use wasmer_middlewares::Metering;

use crate::middlewares::gas_calibration::GasCalibration;
use crate::settings::max_number_of_pages;
use crate::tunable_memory::LimitingTunables;
use crate::GasCosts;

pub(crate) use common::*;
pub(crate) use context::*;
pub(crate) use error::*;

#[derive(Clone)]
pub enum RuntimeModule {
    ASModule((ASModule, Engine)),
}

impl RuntimeModule {
    /// TODO: Dispatch module creation corresponding to the first bytecode byte
    ///
    /// * (1) target AssemblyScript
    /// * (2) TODO: target X
    /// * (_) target AssemblyScript and use the full bytecode
    pub fn new(bytecode: &[u8], limit: u64, gas_costs: GasCosts) -> Result<Self> {
        match bytecode.first() {
            Some(1) => Ok(Self::ASModule(ASModule::new(bytecode, limit, gas_costs)?)),
            Some(_) => Ok(Self::ASModule(ASModule::new(bytecode, limit, gas_costs)?)),
            None => Err(anyhow!("Empty bytecode")),
        }
    }
}

/// An executable runtime module compiled from an AssemblyScript SC
#[derive(Clone)]
pub struct ASModule {
    pub(crate) binary_module: Module,
    pub(crate) init_limit: u64,
}

impl ASModule {
    pub(crate) fn new(bytecode: &[u8], limit: u64, gas_costs: GasCosts) -> Result<(Self, Engine)> {
        let engine = init_engine(limit, gas_costs);
        Ok((
            Self {
                binary_module: Module::new(&engine, bytecode)?,
                init_limit: limit,
            },
            engine,
        ))
    }
}

pub(crate) fn init_engine(limit: u64, gas_costs: GasCosts) -> Engine {
    // Use Cranelift (in opposition to Singlepass)
    // Because of module caching we can run longer compilations to have better optimizations
    // Executions are happening way more often than compilations hence that choice
    let mut compiler_config = Cranelift::new();

    // Canonicalize NaN & turn off sources of potential non-determinism:
    // * threads
    // * SIMD
    // * experimental features
    // References:
    // * https://github.com/WebAssembly/design/blob/390bab47efdb76b600371bcef1ec0ea374aa8c43/Nondeterminism.md
    // * https://github.com/WebAssembly/proposals
    compiler_config.canonicalize_nans(true);
    const FEATURES: Features = Features {
        threads: false,        // non-deterministic
        reference_types: true, // enables externref WASM type and needs to be enabled with bulk_memory
        simd: false,           // non-deterministic
        bulk_memory: true,     // enables the use of buffers in AS
        multi_value: true,     // enables functions and blocks returning multiple values
        tail_call: false,      // experimental
        module_linking: false, // experimental
        multi_memory: false,   // experimental
        memory64: false,       // experimental
        exceptions: false,     // experimental
        relaxed_simd: false,   // experimental
        extended_const: false, // experimental
    };

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

    EngineBuilder::new(compiler_config)
        .set_features(Some(FEATURES))
        .engine()
}

pub(crate) fn init_store(engine: &Engine) -> Result<Store> {
    let base = BaseTunables::for_target(&Target::default());
    let tunables = LimitingTunables::new(base, Pages(max_number_of_pages()));
    let store = Store::new_with_tunables(engine, tunables);
    Ok(store)
}
