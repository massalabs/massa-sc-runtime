use crate::{types::Interface, GasCosts};
use std::{
    collections::HashMap,
    sync::{atomic::AtomicBool, Arc},
};
use wasmer::Global;

use super::Metered;

/// AssemblyScript execution environment.
///
/// Contains the AS ffi env and all the data required to run a module.
#[derive(Clone)]
pub struct ASEnv {
    /// AssemblyScript foreign function interface environment.
    /// Used to interact with AS types.
    ffi_env: as_ffi_bindings::Env,
    /// Set to true after a module execution was instantiated.
    /// ABIs should be disabled in the AssemblyScript `start` function.
    /// It prevents non-deterministic behaviour in the intances creation.
    pub abi_enabled: Arc<AtomicBool>,
    /// Exposed interface functions used by the ABIs and implemented externally.
    /// In `massa/massa-execution-worker` for example.
    pub interface: Box<dyn Interface>,
    /// Remaining metering points in the current execution context.
    pub remaining_points: Option<Global>,
    /// Cumulated exhausted points in the current execution context.
    pub exhausted_points: Option<Global>,
    /// Gas costs of different execution operations.
    gas_costs: GasCosts,
    /// Initially added for gas calibration but unused at the moment.
    param_size_map: HashMap<String, Option<Global>>,
}

impl ASEnv {
    pub fn new(interface: &dyn Interface, gas_costs: GasCosts) -> Self {
        Self {
            ffi_env: Default::default(),
            abi_enabled: Arc::new(AtomicBool::new(false)),
            gas_costs,
            interface: interface.clone_box(),
            remaining_points: None,
            exhausted_points: None,
            param_size_map: Default::default(),
        }
    }
    pub fn get_interface(&self) -> Box<dyn Interface> {
        self.interface.clone()
    }
    pub fn get_ffi_env(&self) -> &as_ffi_bindings::Env {
        &self.ffi_env
    }
    pub fn get_ffi_env_as_mut(&mut self) -> &mut as_ffi_bindings::Env {
        &mut self.ffi_env
    }
}

impl Metered for ASEnv {
    fn get_exhausted_points(&self) -> Option<&Global> {
        self.exhausted_points.as_ref()
    }
    fn get_remaining_points(&self) -> Option<&Global> {
        self.remaining_points.as_ref()
    }
    fn get_gc_param(&self, name: &str) -> Option<&Global> {
        self.param_size_map.get(name)?.as_ref()
    }
    fn get_gas_costs(&self) -> GasCosts {
        self.gas_costs.clone()
    }
}
