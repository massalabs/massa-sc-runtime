use super::abi::*;
use super::env::{get_remaining_points, set_remaining_points, WasmV1Env, Metered};
use crate::types::Response;
use crate::{GasCosts, Interface};
use anyhow::{bail, Result};
use as_ffi_bindings::{BufferPtr, Read as ASRead, Write as ASWrite};
use wasmer::{
    imports, Function, FunctionEnv, Imports, Instance, InstantiationError, Module, Store, Value,
};
use wasmer_middlewares::metering::{self, MeteringPoints};
use wasmer_types::TrapCode;

pub(crate) struct ASContext {
    pub env: WasmV1Env,
    pub module: Module,
}

/// Execution context of an AS module.
///
/// This object handles every execution step apart from:
/// * Module compilation
/// * Engine creation & init
/// * Store creation & init
impl ASContext {
    pub(crate) fn new(
        interface: &dyn Interface,
        binary_module: Module,
        gas_costs: GasCosts,
    ) -> Self {
        Self {
            env: WasmV1Env::new(interface, gas_costs),
            module: binary_module,
        }
    }



    pub(crate) fn execution(
        &self,
        store: &mut Store,
        instance: &Instance,
        function: &str,
        param: &[u8],
    ) -> Result<Response> {
        if cfg!(not(feature = "gas_calibration")) {
            // Sub initial metering cost
            let metering_initial_cost = self.env.get_gas_costs().launch_cost;
            let remaining_gas = get_remaining_points(&self.env, store)?;
            if metering_initial_cost > remaining_gas {
                bail!("Not enough gas to launch the virtual machine")
            }
            set_remaining_points(&self.env, store, remaining_gas - metering_initial_cost)?;
        }

        // Now can exec
        let wasm_func = instance.exports.get_function(function)?;
        let argc = wasm_func.param_arity(store);
        let res = if argc == 0 {
            wasm_func.call(store, &[])
        } else if argc == 1 {
            let param_ptr = *BufferPtr::alloc(&param.to_vec(), self.env.get_ffi_env(), store)?;
            wasm_func.call(store, &[Value::I32(param_ptr.offset() as i32)])
        } else {
            bail!("Unexpected number of parameters in the function called")
        };

        match res {
            Ok(value) => {
                if function.eq(crate::settings::MAIN) {
                    let remaining_gas = if cfg!(feature = "gas_calibration") {
                        Ok(0_u64)
                    } else {
                        get_remaining_points(&self.env, store)
                    };

                    return Ok(Response {
                        ret: Vec::new(), // main return empty vec
                        remaining_gas: remaining_gas?,
                        init_cost: 0,
                    });
                }
                let ret = if let Some(offset) = value.first() {
                    if let Some(offset) = offset.i32() {
                        let buffer_ptr = BufferPtr::new(offset as u32);
                        let memory = instance.exports.get_memory("memory")?;
                        buffer_ptr.read(memory, store)?
                    } else {
                        bail!("Execution wasn't in capacity to read the return value")
                    }
                } else {
                    Vec::new()
                };
                let remaining_gas = if cfg!(feature = "gas_calibration") {
                    Ok(0_u64)
                } else {
                    get_remaining_points(&self.env, store)
                };
                Ok(Response {
                    ret,
                    remaining_gas: remaining_gas?,
                    init_cost: 0,
                })
            }
            Err(error) => bail!(error),
        }
    }

}
