use std::sync::Arc;

use super::{ffi::Ffi, WasmV1Error};
use crate::GasCosts;
use crate::{types::Interface, CondomLimits};
use parking_lot::Mutex;
use wasmer::{AsStoreMut, AsStoreRef, Imports, Instance, InstantiationError, TypedFunction};
use wasmer_middlewares::metering::{self, MeteringPoints};
use wasmer_types::TrapCode;

#[cfg(feature = "execution-trace")]
use crate::AbiTrace;

pub type ABIEnv = Arc<Mutex<Option<ExecutionEnv>>>;

/// Execution environment for ABIs.
#[derive(Clone)]
pub struct ExecutionEnv {
    /// Exposed interface functions used by the ABIs and implemented
    /// externally. In `massa/massa-execution-worker` for example.
    interface: Box<dyn Interface>,
    /// Gas costs of different execution operations.
    gas_costs: GasCosts,
    /// Instance to execute
    pub(crate) instance: Instance,
    /// Memory interface
    ffi: Ffi,
    /// Gas cost of instance creation
    init_gas_cost: u64,
    /// Maximum number of exports
    condom_limits: CondomLimits,

    #[cfg(feature = "execution-trace")]
    pub trace: Vec<AbiTrace>,
}

/// ABI environment giving ABIs access to the interface, gas costs and memory.
impl ExecutionEnv {
    /// Create a new ABI environment.
    pub fn create_instance(
        store: &mut impl AsStoreMut,
        module: &super::WasmV1Module,
        interface: &dyn Interface,
        gas_costs: GasCosts,
        import_object: &Imports,
        condom_limits: CondomLimits,
    ) -> Result<Self, WasmV1Error> {
        // Create the instance
        let instance = match Instance::new(store, &module.binary_module, import_object) {
            Ok(instance) => instance,
            Err(err) => {
                // Filter the error created by the metering middleware when
                // there is not enough gas at initialization
                if let InstantiationError::Start(ref e) = err {
                    if let Some(trap) = e.clone().to_trap() {
                        if trap == TrapCode::UnreachableCodeReached && e.trace().is_empty() {
                            return Err(WasmV1Error::InstanciationError(
                                "Not enough gas, limit reached at instance creation".to_string(),
                            ));
                        }
                    }
                }
                return Err(WasmV1Error::InstanciationError(format!(
                    "Error during instance creation: {}",
                    err
                )));
            }
        };

        // Create FFI for memory access
        let ffi = Ffi::try_new(&instance, store)
            .map_err(|err| WasmV1Error::RuntimeError(format!("Could not create FFI: {}", err)))?;

        // Infer the gas cost of instance creation (_start function call)
        let mut init_gas_cost = 0;
        if cfg!(not(feature = "gas_calibration")) {
            init_gas_cost = match metering::get_remaining_points(store, &instance) {
                MeteringPoints::Remaining(remaining_points) => module
                    .gas_limit_at_compilation
                    .checked_sub(remaining_points)
                    .expect(
                        "Remaining gas after instance creation is higher than the gas limit at compilation",
                    ),
                MeteringPoints::Exhausted => {
                    return Err(WasmV1Error::InstanciationError(
                        "Not enough gas, gas exhausted after instance creation".to_string(),
                    ));
                }
            };
        }

        // Return the environment
        Ok(Self {
            gas_costs,
            interface: interface.clone_box(),
            instance,
            ffi,
            init_gas_cost,
            condom_limits,
            #[cfg(feature = "execution-trace")]
            trace: Default::default(),
        })
    }

    /// Get gas cost of instance creation
    pub fn get_init_gas_cost(&self) -> u64 {
        self.init_gas_cost
    }

    /// Get interface.
    pub fn get_interface(&self) -> &dyn Interface {
        &*self.interface
    }

    /// Get a typed guest function from the instance.
    pub fn get_func(
        &self,
        store: &impl AsStoreRef,
        function_name: &str,
    ) -> Result<TypedFunction<i32, i32>, WasmV1Error> {
        self.instance
            .exports
            .get_typed_function::<i32, i32>(&store, function_name)
            .map_err(|err| {
                WasmV1Error::RuntimeError(format!(
                    "Error getting typed guest function {}: {}",
                    function_name, err
                ))
            })
    }

    /// Try subtracting gas from the metering.
    pub fn try_subtract_gas(
        &self,
        store: &mut impl AsStoreMut,
        gas: u64,
    ) -> Result<(), WasmV1Error> {
        if cfg!(feature = "gas_calibration") {
            return Ok(());
        }

        let remaining = match metering::get_remaining_points(store, &self.instance) {
            metering::MeteringPoints::Remaining(remaining) => remaining,
            metering::MeteringPoints::Exhausted => {
                return Err(WasmV1Error::RuntimeError(
                    "Gas exhausted before ABI call".into(),
                ))
            }
        };
        let new_remaining = match remaining.checked_sub(gas) {
            Some(v) => v,
            None => {
                return Err(WasmV1Error::RuntimeError(
                    "Gas exhausted after ABI call".into(),
                ))
            }
        };
        metering::set_remaining_points(store, &self.instance, new_remaining);
        Ok(())
    }

    /// Get remaining gas.
    pub fn get_remaining_gas(&self, store: &mut impl AsStoreMut) -> u64 {
        if cfg!(feature = "gas_calibration") {
            return u64::MAX;
        }

        match metering::get_remaining_points(store, &self.instance) {
            metering::MeteringPoints::Remaining(remaining) => remaining,
            metering::MeteringPoints::Exhausted => 0,
        }
    }

    /// Set remaining gas.
    pub fn set_remaining_gas(&self, store: &mut impl AsStoreMut, remaining_gas: u64) {
        if cfg!(not(feature = "gas_calibration")) {
            metering::set_remaining_points(store, &self.instance, remaining_gas);
        }
    }

    /// Read buffer from guest memory,
    /// try to deallocate it.
    pub fn take_buffer(
        &self,
        store: &mut impl AsStoreMut,
        offset: i32,
    ) -> Result<Vec<u8>, WasmV1Error> {
        self.ffi.take_buffer(store, offset)
    }

    /// Allocate a buffer into guest memory,
    /// write data into it.
    pub fn create_buffer(
        &self,
        store: &mut impl AsStoreMut,
        data: &[u8],
    ) -> Result<i32, WasmV1Error> {
        self.ffi.create_buffer(store, data)
    }

    /// Get gas costs.
    pub fn get_gas_costs(&self) -> &GasCosts {
        &self.gas_costs
    }

    /// Get the condom limits.
    pub fn get_condom_limits(&self) -> &CondomLimits {
        &self.condom_limits
    }

    /// Get the memory maximum size in bytes
    pub fn get_max_mem_size(&self, store: &mut impl AsStoreMut) -> u64 {
        self.ffi.get_max_mem_size(&store)
    }
}
