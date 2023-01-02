mod as_abi;
mod as_execution;
mod common;

use anyhow::{bail, Result};
use std::sync::Arc;
use wasmer::{wasmparser::Operator, BaseTunables, Pages, Target};
use wasmer::{CompilerConfig, Features, ImportObject, Instance, Module, Store, Universal};
use wasmer_compiler_singlepass::Singlepass;
use wasmer_middlewares::Metering;

use crate::middlewares::gas_calibration::GasCalibration;
use crate::settings::max_number_of_pages;
use crate::tunable_memory::LimitingTunables;
use crate::{Interface, Response};

pub(crate) use as_execution::*;
pub(crate) use common::*;

pub(crate) trait MassaModule {
    fn new(interface: &dyn Interface, store: Store, module: Module) -> Self;
    fn create_vm_instance_and_init_env(&mut self) -> Result<Instance>;
    /// Closure for the execution allowing us to handle a gas error
    fn execution(&self, instance: &Instance, function: &str, param: &[u8]) -> Result<Response>;
    fn resolver(&self) -> ImportObject;
}

pub(crate) fn compile_bytecode(
    interface: &dyn Interface,
    limit: u64,
    bytecode: &[u8],
) -> Result<impl MassaModule> {
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
        let metering = Arc::new(Metering::new(limit, |_: &Operator| -> u64 { 1 }));
        compiler_config.push_middleware(metering);
    }

    let base = BaseTunables::for_target(&Target::default());
    let tunables = LimitingTunables::new(base, Pages(max_number_of_pages()));
    let engine = Universal::new(compiler_config).features(FEATURES).engine();
    let store = Store::new_with_tunables(&engine, tunables);
    let module = Module::new(&store, bytecode)?;

    // Return the Wasmer module
    Ok(ASModule::new(interface, store, module))
}

/// Dispatch module corresponding to the first bytecode.
/// 1: target AssemblyScript
/// 2: todo: another target
/// _: target AssemblyScript and use the full bytecode
pub(crate) fn examine_and_compile_bytecode(
    interface: &dyn Interface,
    limit: u64,
    bytecode: &[u8],
) -> Result<impl MassaModule> {
    if bytecode.is_empty() {
        bail!("error: module is empty")
    }
    Ok(match bytecode[0] {
        1 => compile_bytecode(interface, limit, bytecode)?,
        _ => compile_bytecode(interface, limit, bytecode)?,
    })
}
