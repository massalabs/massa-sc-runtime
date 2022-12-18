mod as_abi;
mod as_execution;
mod common;

use anyhow::{bail, Result};
use std::sync::Arc;
use wasmer::{wasmparser::Operator, BaseTunables, Pages, Target};
use wasmer::{
    CompilerConfig, Features, HostEnvInitError, ImportObject, Instance, InstantiationError, Module,
    Store, Universal,
};
use wasmer_compiler_singlepass::Singlepass;
use wasmer_middlewares::Metering;
use wasmer_types::TrapCode;

use crate::middlewares::gas_calibration::GasCalibration;
use crate::settings::{max_number_of_pages, OPERATOR_COST, GAS_COSTS};
use crate::tunable_memory::LimitingTunables;
use crate::{Interface, Response};

pub(crate) use as_execution::*;
pub(crate) use common::*;

pub(crate) trait MassaModule {
    fn init(interface: &dyn Interface, bytecode: &[u8]) -> Self;
    /// Closure for the execution allowing us to handle a gas error
    fn execution(&self, instance: &Instance, function: &str, param: &[u8]) -> Result<Response>;
    fn resolver(&self, store: &Store) -> ImportObject;
    fn init_with_instance(&mut self, instance: &Instance) -> Result<(), HostEnvInitError>;
    fn get_bytecode(&self) -> &Vec<u8>;
}

/// Create an instance of VM from a module with a given interface, an operation
/// number limit and a webassembly module
pub(crate) fn create_instance(limit: u64, module: &impl MassaModule) -> Result<Instance> {
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
        println!("AURELIEN : GASCOST : {}", *GAS_COSTS);
        println!("AURELIEN : COST : {}", *OPERATOR_COST);
        // Add metering middleware
        let metering = Arc::new(Metering::new(limit, |_: &Operator| -> u64 {
            *OPERATOR_COST as u64
        }));
        compiler_config.push_middleware(metering);
    }

    let base = BaseTunables::for_target(&Target::default());
    let tunables = LimitingTunables::new(base, Pages(max_number_of_pages()));
    let engine = Universal::new(compiler_config).features(FEATURES).engine();
    let store = Store::new_with_tunables(&engine, tunables);

    match Instance::new(
        &Module::new(&store, module.get_bytecode())?,
        &module.resolver(&store),
    ) {
        Ok(i) => Ok(i),
        Err(err) => {
            // We filter the error created by the metering middleware when there is not enough gas at initialization.
            if let InstantiationError::Start(ref e) = err {
                if let Some(trap) = e.clone().to_trap() {
                    if trap == TrapCode::UnreachableCodeReached && e.trace().is_empty() {
                        bail!("RuntimeError: Not enough gas, limit reached at initialization");
                    }
                }
            }
            Err(err.into())
        }
    }
}

/// Dispatch module corresponding to the first bytecode.
/// 1: target AssemblyScript
/// 2: todo: another target
/// _: target AssemblyScript and use the full bytecode
pub(crate) fn get_module(interface: &dyn Interface, bytecode: &[u8]) -> Result<impl MassaModule> {
    if bytecode.is_empty() {
        bail!("error: module is empty")
    }
    Ok(match bytecode[0] {
        1 => ASModule::init(interface, &bytecode[1..]),
        _ => ASModule::init(interface, bytecode),
    })
}
