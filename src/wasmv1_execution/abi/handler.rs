use super::super::env::{ABIEnv, ExecutionEnv};
use crate::{wasmv1_execution::WasmV1Error, CondomLimits, GasCosts};
use std::io::Cursor;
use wasmer::FunctionEnvMut;

/// Macro to handle an ABI call by providing helpers to read arguments, return values
/// Takes the gas cost field name directly to avoid HashMap lookup
#[macro_export]
macro_rules! handle_abi {
    ($gas_field:ident, $store_env:expr, $arg_offset:expr, $func:expr) => {{
        use $crate::wasmv1_execution::WasmV1Error;

        let mut store_env = $store_env;
        // get environment and interface
        let env_mutex = store_env.data().clone();
        let mut env_lock = env_mutex.lock();
        let exec_env = env_lock.as_mut().ok_or_else(|| {
            WasmV1Error::InstanciationError("ABIs cannot be called at initialization time.".into())
        })?;

        // get gas cost directly from struct field
        let gas_cost = exec_env.get_gas_costs().$gas_field;

        // create handler
        let mut handler = $crate::wasmv1_execution::abi::handler::ABIHandler {
            store_env: &mut store_env,
            exec_env,
        };

        // apply gas cost
        if gas_cost > 0 {
            handler.try_subtract_gas(gas_cost)?;
        }

        // read argument
        let arg = handler.read_arg($arg_offset)?;

        // call function
        let func = $func;
        let response = func(&mut handler, arg)?;

        // return value
        handler.return_value(response)
    }};
}

/// Macro to handle an ABI call with raw bytes (cannot rely on encoding)
/// Takes the gas cost field name directly to avoid HashMap lookup
#[macro_export]
macro_rules! handle_abi_raw {
    ($gas_field:ident, $store_env:expr, $arg_offset:expr, $func:expr) => {{
        use $crate::wasmv1_execution::WasmV1Error;

        let mut store_env = $store_env;
        // get environment and interface
        let env_mutex = store_env.data().clone();
        let mut env_lock = env_mutex.lock();
        let exec_env = env_lock.as_mut().ok_or_else(|| {
            WasmV1Error::InstanciationError("ABIs cannot be called at initialization time.".into())
        })?;

        // get gas cost directly from struct field
        let gas_cost = exec_env.get_gas_costs().$gas_field;

        // create handler
        let mut handler = $crate::wasmv1_execution::abi::handler::ABIHandler {
            store_env: &mut store_env,
            exec_env,
        };

        // apply gas cost
        if gas_cost > 0 {
            handler.try_subtract_gas(gas_cost)?;
        }

        // read argument
        let arg: Vec<u8> = handler.read_arg_raw($arg_offset)?;

        // call function
        let func = $func;
        let response = func(&mut handler, arg)?;

        // return value
        handler.return_value_raw(&response)
    }};
}

/// A helper structure to handle ABI calls
pub struct ABIHandler<'a, 'b> {
    pub(crate) store_env: &'b mut FunctionEnvMut<'a, ABIEnv>,
    pub(crate) exec_env: &'b mut ExecutionEnv,
}

impl<'a, 'b> ABIHandler<'a, 'b> {
    /// Read argument
    pub fn read_arg<M>(&mut self, arg_offset: i32) -> Result<M, WasmV1Error>
    where
        M: prost::Message + Default,
    {
        let byte_vec = self
            .exec_env
            .take_buffer(&mut self.store_env, arg_offset)
            .map_err(|err| {
                WasmV1Error::RuntimeError(format!("Could not read ABI argument: {}", err))
            })?;
        M::decode(&mut Cursor::new(&byte_vec)).map_err(|err| {
            WasmV1Error::RuntimeError(format!("Could not deserialize ABI argument: {}", err))
        })
    }

    /// Read argument raw
    /// For use with abort and other function that cannot use protobuf
    pub fn read_arg_raw(&mut self, arg_offset: i32) -> Result<Vec<u8>, WasmV1Error> {
        let byte_vec = self
            .exec_env
            .take_buffer(&mut self.store_env, arg_offset)
            .map_err(|err| {
                WasmV1Error::RuntimeError(format!("Could not read ABI argument: {}", err))
            })?;

        Ok(byte_vec)
    }

    /// Return a value
    pub fn return_value<M>(&mut self, value: M) -> Result<i32, WasmV1Error>
    where
        M: prost::Message,
    {
        let mut buf = Vec::with_capacity(value.encoded_len());
        value.encode(&mut buf).map_err(|err| {
            WasmV1Error::RuntimeError(format!("Could not serialize ABI return value: {}", err))
        })?;
        self.exec_env
            .create_buffer(&mut self.store_env, &buf)
            .map_err(|err| {
                WasmV1Error::RuntimeError(format!("Could not write ABI return value: {}", err))
            })
    }

    /// Return a raw value aka a Vec<u8> any encoding is up to the caller
    pub fn return_value_raw(&mut self, value: &[u8]) -> Result<i32, WasmV1Error> {
        self.exec_env
            .create_buffer(&mut self.store_env, value)
            .map_err(|err| {
                WasmV1Error::RuntimeError(format!("Could not write ABI return value: {}", err))
            })
    }

    /// Try subtracting gas
    pub fn try_subtract_gas(&mut self, gas: u64) -> Result<(), WasmV1Error> {
        self.exec_env
            .try_subtract_gas(&mut self.store_env, gas)
            .map_err(|err| WasmV1Error::RuntimeError(format!("ABI gas error: {}", err)))
    }

    /// Get remaining gas
    pub fn get_remaining_gas(&mut self) -> u64 {
        self.exec_env.get_remaining_gas(&mut self.store_env)
    }

    /// Set remaining gas
    pub fn set_remaining_gas(&mut self, gas: u64) {
        self.exec_env.set_remaining_gas(&mut self.store_env, gas)
    }

    /// Get gas costs structure
    pub fn get_gas_costs(&self) -> &GasCosts {
        self.exec_env.get_gas_costs()
    }

    /// Get condom limits
    pub fn get_condom_limits(&self) -> &CondomLimits {
        self.exec_env.get_condom_limits()
    }

    /// Get the memory maximum size in bytes
    pub fn get_max_mem_size(&mut self) -> u64 {
        self.exec_env.get_max_mem_size(self.store_env)
    }
}
