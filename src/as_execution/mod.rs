mod abi;
mod common;
mod context;
mod error;

use anyhow::{anyhow, Result};
use std::sync::Arc;
use wasmer::{wasmparser::Operator, BaseTunables, EngineBuilder, Pages, Target};
use wasmer::{CompilerConfig, Cranelift, Engine, Features, Module, Store};
use wasmer_compiler_singlepass::Singlepass;
use wasmer_middlewares::Metering;

use crate::middlewares::{dumper::Dumper, gas_calibration::GasCalibration};
use crate::settings::max_number_of_pages;
use crate::tunable_memory::LimitingTunables;
use crate::GasCosts;

pub(crate) use context::*;
pub(crate) use error::*;

#[derive(Clone)]
pub enum RuntimeModule {
    ASModule(ASModule),
}

impl RuntimeModule {
    /// TODO: Dispatch module creation corresponding to the first bytecode byte
    ///
    /// * (1) TODO: target AssemblyScript (remove ident)
    /// * (2) TODO: target X
    /// * (_) target AssemblyScript
    pub fn new(
        bytecode: &[u8],
        limit: u64,
        gas_costs: GasCosts,
        cache_compatible: bool,
    ) -> Result<Self> {
        match bytecode.first() {
            Some(_) => Ok(Self::ASModule(ASModule::new(
                bytecode,
                limit,
                gas_costs,
                cache_compatible,
            )?)),
            None => Err(anyhow!("Empty bytecode")),
        }
    }

    // NOTE: set a module identifier for other types of sub modules
    // distinction between runtime module ident and sub module ident must be clear
    // if the serialization process becomes too complex use NOM
    pub fn serialize(&self) -> Result<Vec<u8>> {
        let ser = match self {
            RuntimeModule::ASModule(module) => module.serialize()?,
        };
        Ok(ser)
    }

    // NOTE: only deserialize from ASModule for now
    // make a distinction based on the runtime module identifier byte
    // see serialize note
    pub fn deserialize(ser_module: &[u8], limit: u64, gas_costs: GasCosts) -> Result<Self> {
        let deser = RuntimeModule::ASModule(ASModule::deserialize(ser_module, limit, gas_costs)?);
        Ok(deser)
    }
}

/// An executable runtime module compiled from an AssemblyScript SC
#[derive(Clone)]
pub struct ASModule {
    pub(crate) binary_module: Module,
    pub(crate) initial_limit: u64,
    pub cache_compatible: bool,
    // NOTE: might need to move the engine back out
    #[allow(dead_code)]
    pub(crate) engine: Engine,
}

impl ASModule {
    pub(crate) fn new(
        bytecode: &[u8],
        limit: u64,
        gas_costs: GasCosts,
        cache_compatible: bool,
    ) -> Result<Self> {
        let engine = if cache_compatible {
            init_cl_engine(limit, gas_costs)
        } else {
            init_sp_engine(limit, gas_costs)
        };
        Ok(Self {
            binary_module: Module::new(&engine, bytecode)?,
            initial_limit: limit,
            cache_compatible,
            engine,
        })
    }

    pub fn serialize(&self) -> Result<Vec<u8>> {
        let mut ser_module = self.binary_module.serialize()?.to_vec();
        ser_module.insert(0, u8::from(self.cache_compatible));
        Ok(ser_module)
    }

    pub fn deserialize(ser_module: &[u8], limit: u64, gas_costs: GasCosts) -> Result<Self> {
        // NOTE: move this check to serialize
        // Deserialization is only meant for Cranelift modules
        let engine = match ser_module.first() {
            Some(1) => init_cl_engine(limit, gas_costs),
            Some(_) => panic!("invalid or unsupported identifier byte"),
            None => panic!("empty serialized module"),
        };
        let store = init_store(&engine)?;
        // Unsafe because code injection is possible
        // That's not an issue in our case since we only deserialize modules we trust
        let module = unsafe { Module::deserialize(&store, ser_module)? };
        Ok(ASModule {
            binary_module: module,
            initial_limit: limit,
            cache_compatible: true,
            engine,
        })
    }
}

pub(crate) fn init_sp_engine(limit: u64, gas_costs: GasCosts) -> Engine {
    // Singlepass is used to compile arbitrary bytecode.
    //
    // Reference:
    // * https://docs.rs/wasmer-compiler-singlepass/latest/wasmer_compiler_singlepass/
    let mut compiler_config = Singlepass::new();

    // Canonicalize NaN, turn off all sources of non-determinism.
    //
    // References:
    // * https://github.com/webassembly/bulk-memory-operations
    // * https://github.com/WebAssembly/design/blob/390bab47efdb76b600371bcef1ec0ea374aa8c43/Nondeterminism.md
    // * https://github.com/WebAssembly/proposals
    //
    // TLDR: Turn off every feature except for `bulk_memory`.
    compiler_config.canonicalize_nans(true);
    const FEATURES: Features = Features {
        threads: false,         // non-deterministic
        reference_types: false, // not supported by Singlepass
        simd: false,            // non-deterministic
        bulk_memory: true,      // enables the use of buffers in AS
        multi_value: false,     // not supported by Singlepass
        tail_call: false,       // experimental
        module_linking: false,  // experimental
        multi_memory: false,    // experimental
        memory64: false,        // experimental
        exceptions: false,      // experimental
        relaxed_simd: false,    // experimental
        extended_const: false,  // experimental
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

pub(crate) fn init_cl_engine(limit: u64, gas_costs: GasCosts) -> Engine {
    // Cranelift is used to compile bytecode that will be cached.
    //
    // Reference:
    // * https://docs.rs/wasmer-compiler-cranelift/latest/wasmer_compiler_cranelift/
    let mut compiler_config = Cranelift::new();

    // Canonicalize NaN, turn off all sources of non-determinism.
    //
    // References:
    // * https://github.com/webassembly/bulk-memory-operations
    // * https://github.com/WebAssembly/design/blob/390bab47efdb76b600371bcef1ec0ea374aa8c43/Nondeterminism.md
    // * https://github.com/WebAssembly/proposals
    //
    // TLDR: Turn off every feature except for `bulk_memory`.
    compiler_config.canonicalize_nans(true);
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
