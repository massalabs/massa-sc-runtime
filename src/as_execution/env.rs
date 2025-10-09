use super::{abi_bail, ABIResult};
use crate::{types::Interface, CondomLimits};

#[cfg(feature = "execution-trace")]
use crate::types::AbiTrace;

use crate::GasCosts;
use std::{
    collections::HashMap,
    sync::{atomic::AtomicBool, Arc},
};
use wasmer::{AsStoreMut, Global};

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
    /// Exposed interface functions used by the ABIs and implemented
    /// externally. In `massa/massa-execution-worker` for example.
    pub interface: Box<dyn Interface>,
    /// Remaining metering points in the current execution context.
    pub remaining_points: Option<Global>,
    /// Cumulated exhausted points in the current execution context.
    pub exhausted_points: Option<Global>,
    /// Gas costs of different execution operations.
    gas_costs: GasCosts,
    /// Maximum number of exports
    condom_limits: CondomLimits,
    /// Initially added for gas calibration but unused at the moment.
    #[allow(dead_code)]
    param_size_map: HashMap<String, Option<Global>>,
    #[cfg(feature = "execution-trace")]
    pub trace: Vec<AbiTrace>,
}

impl ASEnv {
    pub fn new(
        interface: &dyn Interface,
        gas_costs: GasCosts,
        condom_limits: CondomLimits,
    ) -> Self {
        Self {
            ffi_env: Default::default(),
            abi_enabled: Arc::new(AtomicBool::new(false)),
            gas_costs,
            interface: interface.clone_box(),
            remaining_points: None,
            exhausted_points: None,
            param_size_map: Default::default(),
            condom_limits,
            #[cfg(feature = "execution-trace")]
            trace: Default::default(),
        }
    }
    pub fn get_interface(&self) -> &dyn Interface {
        &*self.interface
    }
    pub fn get_ffi_env(&self) -> &as_ffi_bindings::Env {
        &self.ffi_env
    }
    pub fn get_ffi_env_as_mut(&mut self) -> &mut as_ffi_bindings::Env {
        &mut self.ffi_env
    }
    pub fn get_gas_costs(&self) -> &GasCosts {
        &self.gas_costs
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
    fn get_condom_limits(&self) -> &CondomLimits {
        &self.condom_limits
    }
}

/// Trait describing a metered object.
///
/// An object implementing this trait can track the execution consumption.
pub(crate) trait Metered {
    fn get_exhausted_points(&self) -> Option<&Global>;
    fn get_remaining_points(&self) -> Option<&Global>;
    #[allow(dead_code)]
    fn get_gc_param(&self, name: &str) -> Option<&Global>;
    fn get_condom_limits(&self) -> &CondomLimits;
}

/// Get remaining metering points.
/// Should be equivalent to:
/// https://github.com/wasmerio/wasmer/blob/8f2e49d52823cb7704d93683ce798aa84b6928c8/lib/middlewares/src/metering.rs#L293
pub(crate) fn get_remaining_points(
    env: &impl Metered,
    store: &mut impl AsStoreMut,
) -> ABIResult<u64> {
    if cfg!(feature = "gas_calibration") {
        Ok(u64::MAX)
    } else {
        match env.get_exhausted_points().as_ref() {
            Some(exhausted_points) => {
                match exhausted_points.get(store).try_into() {
                    // Using i32 here because it's the type used internally by
                    // wasmer for exhausted.
                    Ok::<i32, _>(exhausted) if exhausted > 0 => return Ok(0),
                    Ok::<i32, _>(_) => (),
                    Err(_) => abi_bail!("exhausted_points has wrong type"),
                }
            }
            None => abi_bail!("Lost reference to exhausted_points"),
        };
        match env.get_remaining_points().as_ref() {
            Some(remaining_points) => match remaining_points.get(store).try_into() {
                Ok::<u64, _>(remaining) => Ok(remaining),
                Err(_) => abi_bail!("remaining_points has wrong type"),
            },
            None => abi_bail!("Lost reference to remaining_points"),
        }
    }
}

/// Set remaining metering points.
/// Should be equivalent to:
/// https://github.com/wasmerio/wasmer/blob/8f2e49d52823cb7704d93683ce798aa84b6928c8/lib/middlewares/src/metering.rs#L343
pub(crate) fn set_remaining_points(
    env: &impl Metered,
    store: &mut impl AsStoreMut,
    points: u64,
) -> ABIResult<()> {
    if cfg!(not(feature = "gas_calibration")) {
        match env.get_remaining_points().as_ref() {
            Some(remaining_points) => {
                if remaining_points.set(store, points.into()).is_err() {
                    abi_bail!("Can't set remaining_points");
                }
            }
            None => abi_bail!("Lost reference to remaining_points"),
        };
        match env.get_exhausted_points().as_ref() {
            Some(exhausted_points) => {
                if exhausted_points.set(store, 0i32.into()).is_err() {
                    abi_bail!("Can't set exhausted_points")
                }
            }
            None => abi_bail!("Lost reference to exhausted_points"),
        };
    }
    Ok(())
}

/// Optimized version that works with pre-extracted Global handles
/// This avoids extra ctx.data() calls
pub(crate) fn sub_remaining_gas_with_globals(
    remaining_points: &wasmer::Global,
    exhausted_points: &wasmer::Global,
    ctx: &mut impl AsStoreMut,
    gas: u64,
) -> ABIResult<()> {
    if cfg!(feature = "gas_calibration") {
        return Ok(());
    }

    // Check exhausted points
    match exhausted_points.get(ctx).try_into() {
        Ok::<i32, _>(exhausted) if exhausted > 0 => {
            abi_bail!("Out of gas");
        }
        Ok::<i32, _>(_) => (),
        Err(_) => abi_bail!("exhausted_points has wrong type"),
    }

    // Get remaining points
    let remaining_gas: u64 = match remaining_points.get(ctx).try_into() {
        Ok(remaining) => remaining,
        Err(_) => abi_bail!("remaining_points has wrong type"),
    };

    // Subtract gas
    if let Some(new_remaining) = remaining_gas.checked_sub(gas) {
        if remaining_points.set(ctx, new_remaining.into()).is_err() {
            abi_bail!("Can't set remaining_points");
        }
        if exhausted_points.set(ctx, 0i32.into()).is_err() {
            abi_bail!("Can't set exhausted_points");
        }
    } else {
        abi_bail!("Out of gas")
    }
    Ok(())
}
