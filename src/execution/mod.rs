mod as_abi;
mod as_execution;
mod common;

use anyhow::{bail, Result};
use wasmer::{wasmparser::Operator, BaseTunables, Pages, Target};
use wasmer::{
    CompilerConfig, Features, HostEnvInitError, ImportObject, Instance, Module, Store, Universal,
};
use wasmer_compiler_singlepass::Singlepass;
use wasmer_middlewares::Metering;

use crate::settings::max_number_of_pages;
use crate::tunable_memory::LimitingTunables;
use crate::{Interface, Response};

use std::sync::Arc;

pub(crate) use as_execution::*;
pub(crate) use common::*;
pub(crate) trait MassaModule {
    fn init(interface: &dyn Interface, bytecode: &[u8]) -> Self;
    /// Closure for the execution allowing us to handle a gas error
    fn execution(&self, instance: &Instance, function: &str, param: &str) -> Result<Response>;
    fn resolver(&self, store: &Store) -> ImportObject;
    fn init_with_instance(&mut self, instance: &Instance) -> Result<(), HostEnvInitError>;
    fn get_bytecode(&self) -> &Vec<u8>;
}

/// Create an instance of VM from a module with a given interface, an operation
/// number limit and a webassembly module
pub(crate) fn create_instance(
    limit: u64,
    bytecode: &[u8],
    module: &impl MassaModule,
) -> Result<Instance> {
    // We use the Singlepass compiler because it is fast and adapted to blockchains
    // See https://docs.rs/wasmer-compiler-singlepass/latest/wasmer_compiler_singlepass/
    let mut compiler_config = Singlepass::new();

    // Turning-off sources of potential non-determinism,
    // see https://github.com/WebAssembly/design/blob/037c6fe94151eb13e30d174f5f7ce851be0a573e/Nondeterminism.md

    // Turning-off in the compiler:

    // Canonicalize NaN.
    compiler_config.canonicalize_nans(true);

    // enable stack check
    compiler_config.enable_stack_check(true);

    // Turning-off in wasmer feature flags:
    let mut features = Features::new();

    // Disable threads.
    features.threads(false);

    // Turn-off experimental SIMD feature.
    features.simd(false);

    // Turn-off multivalue, because it is not supported for Singlepass(and it's true by default).
    features.multi_value(false);

    // Add metering middleware
    let metering = Arc::new(Metering::new(limit, |_: &Operator| -> u64 { 1 }));
    compiler_config.push_middleware(metering);

    let base = BaseTunables::for_target(&Target::default());
    let tunables = LimitingTunables::new(base, Pages(max_number_of_pages()));
    let engine = Universal::new(compiler_config).features(features).engine();
    let store = Store::new_with_tunables(&engine, tunables);

    Ok(Instance::new(
        &Module::new(&store, &bytecode)?,
        &module.resolver(&store),
    )?)
}

/// Dispatch module corresponding to the first bytecode.
/// 0: target AssemblyScript
/// 1: todo: another target
/// _: target AssemblyScript and use the full bytecode
pub(crate) fn get_module(interface: &dyn Interface, bytecode: &[u8]) -> Result<impl MassaModule> {
    if bytecode.is_empty() {
        bail!("error: module is empty")
    }
    Ok(match bytecode[0] {
        0 => ASModule::init(interface, &bytecode[1..]),
        _ => ASModule::init(interface, bytecode),
    })
}
