mod as_abi;
mod as_execution;
mod cache;
mod common;

use anyhow::Result;
use std::sync::Arc;
use wasmer::{wasmparser::Operator, BaseTunables, EngineBuilder, Pages, Target};
use wasmer::{CompilerConfig, Engine, Features, Module, Store};
use wasmer_compiler_singlepass::Singlepass;
use wasmer_middlewares::Metering;

use crate::middlewares::gas_calibration::GasCalibration;
use crate::settings::max_number_of_pages;
use crate::tunable_memory::LimitingTunables;
use crate::GasCosts;

pub(crate) use as_execution::*;
pub use cache::ModuleCache;
pub(crate) use common::*;

pub struct RuntimeModule(ASModule);

impl RuntimeModule {
    /// TODO: Dispatch module creation corresponding to the first bytecode byte
    /// 
    /// * (1) target AssemblyScript
    /// * (2) TODO: target X
    /// * (_) target AssemblyScript and use the full bytecode
    pub fn new(bytecode: &[u8], limit: u64, gas_costs: GasCosts) -> Result<Self> {
        Ok(Self(ASModule::new(bytecode, limit, gas_costs)?))
    }
}

struct ASModule {
    binary_module: Module,
    engine: Engine,
}

impl ASModule {
    pub fn new(bytecode: &[u8], limit: u64, gas_costs: GasCosts) -> Result<Self> {
        let engine = init_engine(limit, gas_costs)?;
        Ok(Self {
            binary_module: Module::new(&engine, bytecode)?,
            engine,
        })
    }
}

pub(crate) fn init_engine(limit: u64, gas_costs: GasCosts) -> Result<Engine> {
    // We use the Singlepass compiler because it is fast and adapted to blockchains
    // See https://docs.rs/wasmer-compiler-singlepass/latest/wasmer_compiler_singlepass/
    let mut compiler_config = Singlepass::new();

    // Turning-off sources of potential non-determinism,
    // see https://github.com/WebAssembly/design/blob/037c6fe94151eb13e30d174f5f7ce851be0a573e/Nondeterminism.md

    // Turning-off in the compiler:

    // Canonicalize NaN.
    compiler_config.canonicalize_nans(true);

    // Default: Turning-off all wasmer feature flags
    // Exception(s):
    // * bulk_memory:
    //   * https://docs.rs/wasmer/latest/wasmer/struct.Features.html: now fully standardized - wasm 2.0
    //   * See also: https://github.com/paritytech/substrate/issues/12216
    const FEATURES: Features = Features {
        threads: false, // disable threads
        reference_types: false,
        simd: false,           // turn off experimental SIMD feature
        bulk_memory: true,     // enabled in order to use ArrayBuffer in AS
        multi_value: false,    // turn off multi value, not support for SinglePass (default: true)
        tail_call: false,      // experimental
        module_linking: false, // experimental
        multi_memory: false,   // experimental
        memory64: false,       // experimental
        exceptions: false,
        relaxed_simd: false, // experimental
        extended_const: false,
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

    Ok(EngineBuilder::new(compiler_config)
        .set_features(Some(FEATURES))
        .engine())
}

pub(crate) fn init_store(engine: &Engine) -> Result<Store> {
    let base = BaseTunables::for_target(&Target::default());
    let tunables = LimitingTunables::new(base, Pages(max_number_of_pages()));
    let store = Store::new_with_tunables(engine, tunables);
    Ok(store)
}
