use crate::{types::Interface, GasCosts};
use parking_lot::RwLock;
use std::{collections::HashMap, sync::Arc};
use wasmer::Global;

use super::Metered;

#[derive(Clone)]
pub struct ASEnv {
    ffi_env: as_ffi_bindings::Env,
    pub abi_enabled: Arc<RwLock<bool>>,
    pub interface: Box<dyn Interface>,
    pub remaining_points: Option<Global>,
    pub exhausted_points: Option<Global>,
    param_size_map: HashMap<String, Option<Global>>,
    gas_costs: GasCosts,
}

impl ASEnv {
    pub fn new(interface: &dyn Interface, gas_costs: GasCosts) -> Self {
        Self {
            ffi_env: Default::default(),
            abi_enabled: Arc::new(RwLock::new(false)),
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
