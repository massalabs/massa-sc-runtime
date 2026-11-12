//! WasmV1 modules are no longer executed: the runtime refuses them from massa
//! MIP-0002 (execution version 2) on, and MIP-0002 is active on mainnet.
//!
//! Only their compilation is kept, because it is still consensus-observable:
//! bytecode that starts with the WasmV1 module id is still compiled and cached,
//! a compilation failure is reported as such, and `function_exists` still
//! answers from the module exports. Removing it would change the result of
//! `function_exists`, `call` or `local_call` on an address holding WasmV1
//! bytecode, which anyone can deploy.

mod error;

use crate::error::VMResult;
use crate::execution::Compiler;
use crate::middlewares::condom::{check_code_limits, CondomMiddleware};
use crate::middlewares::gas_calibration::GasCalibration;
use crate::settings::max_number_of_pages;
use crate::tunable_memory::LimitingTunables;
use crate::{CondomLimits, GasCosts};
pub(crate) use error::*;
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

/// A compiled WasmV1 module. It can be cached and queried for its exports, not executed.
#[derive(Clone)]
pub struct WasmV1Module {
    pub(crate) binary_module: Module,
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
        check_code_limits(bytecode, &condom_limits).map_err(|e| {
            WasmV1Error::InstanciationError(format!("Could not compile bytecode: {}", e))
        })?;
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
