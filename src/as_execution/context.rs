use super::abi::*;
use super::env::{get_remaining_points, set_remaining_points, ASEnv, Metered};
use crate::error::{vm_bail, VMResult};
use crate::types::Response;
use crate::{CondomLimits, GasCosts, Interface};
use as_ffi_bindings::{BufferPtr, Read as ASRead, Write as ASWrite};
use wasmer::{
    imports, Function, FunctionEnv, Imports, Instance, InstantiationError, Module, Store, Value,
};
use wasmer_middlewares::metering::{self, MeteringPoints};
use wasmer_types::TrapCode;

pub(crate) struct ASContext {
    pub env: ASEnv,
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
        condom_limits: CondomLimits,
    ) -> Self {
        Self {
            env: ASEnv::new(interface, gas_costs, condom_limits),
            module: binary_module,
        }
    }

    /// Create a VM instance from the current module
    pub(crate) fn create_vm_instance_and_init_env(
        &mut self,
        store: &mut Store,
    ) -> VMResult<(Instance, FunctionEnv<ASEnv>, u64)> {
        let (imports, mut fenv) = self.resolver(store);
        match Instance::new(store, &self.module, &imports) {
            Ok(instance) => {
                self.init_with_instance(store, &instance, &mut fenv)?;
                let post_init_points = if cfg!(not(feature = "gas_calibration")) {
                    if let MeteringPoints::Remaining(points) =
                        metering::get_remaining_points(store, &instance)
                    {
                        points
                    } else {
                        0
                    }
                } else {
                    0
                };
                self.env
                    .abi_enabled
                    .store(true, std::sync::atomic::Ordering::Relaxed);
                Ok((instance, fenv, post_init_points))
            }
            Err(err) => {
                // Filter the error created by the metering middleware when
                // there is not enough gas at initialization
                if let InstantiationError::Start(ref e) = err {
                    if let Some(trap) = e.clone().to_trap() {
                        if trap == TrapCode::UnreachableCodeReached && e.trace().is_empty() {
                            vm_bail!("Not enough gas, limit reached at initialization");
                        }
                    }
                }
                Err(err.into())
            }
        }
    }

    pub(crate) fn execution(
        &self,
        store: &mut Store,
        instance: &Instance,
        function: &str,
        param: &[u8],
    ) -> VMResult<Response> {
        if cfg!(not(feature = "gas_calibration")) {
            // Sub initial metering cost
            let metering_initial_cost = self.env.get_gas_costs().launch_cost;
            let remaining_gas = get_remaining_points(&self.env, store)?;
            if metering_initial_cost > remaining_gas {
                vm_bail!("Not enough gas to launch the virtual machine")
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
            vm_bail!("Unexpected number of parameters in the function called")
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
                        init_gas_cost: 0,
                        #[cfg(feature = "execution-trace")]
                        trace: Default::default(),
                    });
                }
                let ret = if let Some(offset) = value.first() {
                    if let Some(offset) = offset.i32() {
                        let buffer_ptr = BufferPtr::new(offset as u32);
                        let memory = instance.exports.get_memory("memory")?;
                        buffer_ptr.read(memory, store)?
                    } else {
                        vm_bail!("Execution wasn't in capacity to read the return value")
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
                    init_gas_cost: 0,
                    #[cfg(feature = "execution-trace")]
                    trace: Default::default(),
                })
            }
            Err(error) => Err(error.into()),
        }
    }

    fn init_with_instance(
        &mut self,
        store: &mut Store,
        instance: &Instance,
        fenv: &mut FunctionEnv<ASEnv>,
    ) -> VMResult<()> {
        let memory = instance.exports.get_memory("memory")?;

        // NOTE: only add functions (__new, ...) if these exists in wasm/wat
        // files       so we can still exec some very basic wat files
        let fn_new = instance
            .exports
            .get_typed_function::<(i32, i32), i32>(&store, "__new")
            .ok();
        let fn_pin = instance
            .exports
            .get_typed_function::<i32, i32>(&store, "__pin")
            .ok();
        let fn_unpin = instance
            .exports
            .get_typed_function::<i32, ()>(&store, "__unpin")
            .ok();
        let fn_collect = instance
            .exports
            .get_typed_function::<(), ()>(&store, "__collect")
            .ok();

        fenv.as_mut(store).get_ffi_env_as_mut().init_with(
            Some(memory.clone()),
            fn_new.clone(),
            fn_pin.clone(),
            fn_unpin.clone(),
            fn_collect.clone(),
        );

        // Update self.env as well
        self.env.get_ffi_env_as_mut().init_with(
            Some(memory.clone()),
            fn_new,
            fn_pin,
            fn_unpin,
            fn_collect,
        );

        // Metering counters
        if cfg!(not(feature = "gas_calibration")) {
            let g_1 = instance
                .exports
                .get_global("wasmer_metering_remaining_points")?
                .clone();
            fenv.as_mut(store).remaining_points = Some(g_1.clone());
            let g_2 = instance
                .exports
                .get_global("wasmer_metering_points_exhausted")?
                .clone();
            fenv.as_mut(store).exhausted_points = Some(g_2.clone());

            self.env.remaining_points = Some(g_1);
            self.env.exhausted_points = Some(g_2);
        }

        Ok(())
    }

    pub(crate) fn resolver(&self, store: &mut Store) -> (Imports, FunctionEnv<ASEnv>) {
        let fenv = FunctionEnv::new(store, self.env.clone());

        let version = self.env.interface.get_interface_version().unwrap_or(0);

        let mut imports = imports! {
            "env" => {
                // Needed by WASM generated by AssemblyScript
                "abort" =>  Function::new_typed_with_env(store, &fenv, assembly_script_abort),
                "seed" => Function::new_typed_with_env(store, &fenv, assembly_script_seed),
                "Date.now" =>  Function::new_typed_with_env(store, &fenv, assembly_script_date_now),
                "console.log" =>  Function::new_typed_with_env(store, &fenv, assembly_script_console_log),
                "console.info" =>  Function::new_typed_with_env(store, &fenv, assembly_script_console_info),
                "console.warn" =>  Function::new_typed_with_env(store, &fenv, assembly_script_console_warn),
                "console.error" =>  Function::new_typed_with_env(store, &fenv, assembly_script_console_error),
                "console.debug" =>  Function::new_typed_with_env(store, &fenv, assembly_script_console_debug),
                "trace" =>  Function::new_typed_with_env(store, &fenv, assembly_script_trace),
                "process.exit" =>  Function::new_typed_with_env(store, &fenv, assembly_script_process_exit),
            },
            "massa" => {
                "assembly_script_print" => Function::new_typed_with_env(store, &fenv, assembly_script_print),
                "assembly_script_call" => Function::new_typed_with_env(store, &fenv, assembly_script_call),
                "assembly_script_get_remaining_gas" => Function::new_typed_with_env(store, &fenv, assembly_script_get_remaining_gas),
                "assembly_script_create_sc" => Function::new_typed_with_env(store, &fenv, assembly_script_create_sc),
                "assembly_script_set_data" => Function::new_typed_with_env(store, &fenv, assembly_script_set_data),
                "assembly_script_set_data_for" => Function::new_typed_with_env(store, &fenv, assembly_script_set_data_for),
                "assembly_script_get_data" => Function::new_typed_with_env(store, &fenv, assembly_script_get_data),
                "assembly_script_get_data_for" => Function::new_typed_with_env(store, &fenv, assembly_script_get_data_for),
                "assembly_script_delete_data" => Function::new_typed_with_env(store, &fenv, assembly_script_delete_data),
                "assembly_script_delete_data_for" => Function::new_typed_with_env(store, &fenv, assembly_script_delete_data_for),
                "assembly_script_append_data" => Function::new_typed_with_env(store, &fenv, assembly_script_append_data),
                "assembly_script_append_data_for" => Function::new_typed_with_env(store, &fenv, assembly_script_append_data_for),
                "assembly_script_has_data" => Function::new_typed_with_env(store, &fenv, assembly_script_has_data),
                "assembly_script_has_data_for" => Function::new_typed_with_env(store, &fenv, assembly_script_has_data_for),
                "assembly_script_get_owned_addresses" => Function::new_typed_with_env(store, &fenv, assembly_script_get_owned_addresses),
                "assembly_script_get_call_stack" => Function::new_typed_with_env(store, &fenv, assembly_script_get_call_stack),
                "assembly_script_generate_event" => Function::new_typed_with_env(store, &fenv, assembly_script_generate_event),
                "assembly_script_transfer_coins" => Function::new_typed_with_env(store, &fenv, assembly_script_transfer_coins),
                "assembly_script_transfer_coins_for" => Function::new_typed_with_env(store, &fenv, assembly_script_transfer_coins_for),
                "assembly_script_get_balance" => Function::new_typed_with_env(store, &fenv, assembly_script_get_balance),
                "assembly_script_get_balance_for" => Function::new_typed_with_env(store, &fenv, assembly_script_get_balance_for),
                "assembly_script_hash" => Function::new_typed_with_env(store, &fenv, assembly_script_hash),
                "assembly_script_hash_sha256" =>  Function::new_typed_with_env(store, &fenv, assembly_script_hash_sha256),
                "assembly_script_keccak256_hash" =>  Function::new_typed_with_env(store, &fenv, assembly_script_keccak256_hash),
                "assembly_script_signature_verify" => Function::new_typed_with_env(store, &fenv, assembly_script_signature_verify),
                "assembly_script_evm_signature_verify" => Function::new_typed_with_env(store, &fenv, assembly_script_evm_signature_verify),
                "assembly_script_evm_get_address_from_pubkey" => Function::new_typed_with_env(store, &fenv, assembly_script_evm_get_address_from_pubkey),
                "assembly_script_evm_get_pubkey_from_signature" => Function::new_typed_with_env(store, &fenv, assembly_script_evm_get_pubkey_from_signature),
                "assembly_script_is_address_eoa" => Function::new_typed_with_env(store, &fenv, assembly_script_is_address_eoa),
                "assembly_script_address_from_public_key" => Function::new_typed_with_env(store, &fenv, assembly_script_address_from_public_key),
                "assembly_script_validate_address" => Function::new_typed_with_env(store, &fenv, assembly_script_validate_address),
                "assembly_script_unsafe_random" => Function::new_typed_with_env(store, &fenv, assembly_script_unsafe_random),
                "assembly_script_get_call_coins" => Function::new_typed_with_env(store, &fenv, assembly_script_get_call_coins),
                "assembly_script_get_time" => Function::new_typed_with_env(store, &fenv, assembly_script_get_time),
                "assembly_script_send_message" => Function::new_typed_with_env(store, &fenv, assembly_script_send_message),
                "assembly_script_get_origin_operation_id" => Function::new_typed_with_env(store, &fenv, assembly_script_get_origin_operation_id),
                "assembly_script_get_current_period" => Function::new_typed_with_env(store, &fenv, assembly_script_get_current_period),
                "assembly_script_get_current_thread" => Function::new_typed_with_env(store, &fenv, assembly_script_get_current_thread),
                "assembly_script_set_bytecode" => Function::new_typed_with_env(store, &fenv, assembly_script_set_bytecode),
                "assembly_script_set_bytecode_for" => Function::new_typed_with_env(store, &fenv, assembly_script_set_bytecode_for),
                "assembly_script_get_op_keys" => Function::new_typed_with_env(store, &fenv, assembly_script_get_op_keys),
                "assembly_script_get_op_keys_prefix" => Function::new_typed_with_env(store, &fenv, assembly_script_get_op_keys_prefix),
                "assembly_script_get_keys" => Function::new_typed_with_env(store, &fenv, assembly_script_get_keys),
                "assembly_script_get_keys_for" => Function::new_typed_with_env(store, &fenv, assembly_script_get_keys_for),
                "assembly_script_has_op_key" => Function::new_typed_with_env(store, &fenv, assembly_script_has_op_key),
                "assembly_script_get_op_data" => Function::new_typed_with_env(store, &fenv, assembly_script_get_op_data),
                "assembly_script_get_bytecode" => Function::new_typed_with_env(store, &fenv, assembly_script_get_bytecode),
                "assembly_script_get_bytecode_for" => Function::new_typed_with_env(store, &fenv, assembly_script_get_bytecode_for),
                "assembly_script_local_call" => Function::new_typed_with_env(store, &fenv, assembly_script_local_call),
                "assembly_script_local_execution" => Function::new_typed_with_env(store, &fenv, assembly_script_local_execution),
                "assembly_script_caller_has_write_access" => Function::new_typed_with_env(store, &fenv, assembly_script_caller_has_write_access),
                "assembly_script_function_exists" => Function::new_typed_with_env(store, &fenv, assembly_script_function_exists),
                "assembly_script_chain_id" => Function::new_typed_with_env(store, &fenv, assembly_script_chain_id),
            },
        };

        if version > 0 {
            imports.extend(&imports! {"massa" => {
                "assembly_script_get_deferred_call_quote" => Function::new_typed_with_env(store, &fenv, assembly_script_get_deferred_call_quote),
                "assembly_script_deferred_call_register" => Function::new_typed_with_env(store, &fenv, assembly_script_deferred_call_register),
                "assembly_script_deferred_call_exists" => Function::new_typed_with_env(store, &fenv, assembly_script_deferred_call_exists),
                "assembly_script_deferred_call_cancel" => Function::new_typed_with_env(store, &fenv, assembly_script_deferred_call_cancel),
            }});
        }

        (imports, fenv)
    }
}
