use super::{
    super::env::{ABIEnv, ExecutionEnv},
    proto,
};
use crate::{wasmv1_execution::WasmV1Error, GasCosts, Interface};
use std::io::Cursor;
use wasmer::FunctionEnvMut;

/// Handle an ABI call by providing helpers to read arguments, return values, and so on
pub fn handle_abi<F, Req, Resp>(
    abi_name: &str,
    mut store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
    func: F,
) -> Result<i32, WasmV1Error>
where
    F: FnOnce(&mut ABIHandler, Req) -> Result<Resp, WasmV1Error>,
    Req: proto::Message + Default,
    Resp: proto::Message,
{
    // get environment and interface
    let env_mutex = store_env.data().clone();
    let env_lock = env_mutex.lock();
    let exec_env = env_lock.as_ref().ok_or_else(|| {
        WasmV1Error::InstanciationError("ABIs cannot be called at initialization time.".into())
    })?;
    let interface = &**exec_env.get_interface();

    // create handler
    let mut handler = ABIHandler {
        store_env: &mut store_env,
        exec_env,
        interface,
    };

    // apply gas cost
    let gas_cost = handler.get_gas_cost(abi_name);
    if gas_cost > 0 {
        handler.try_subtract_gas(gas_cost)?;
    }

    // read argument
    let arg: Req = handler.read_arg(arg_offset)?;

    // call function
    let response = func(&mut handler, arg)?;

    // return value
    handler.return_value(response)
}

/// A helper structure to handle ABI calls
pub struct ABIHandler<'a, 'b> {
    store_env: &'b mut FunctionEnvMut<'a, ABIEnv>,
    exec_env: &'b ExecutionEnv,
    pub interface: &'b dyn Interface,
}

impl<'a, 'b> ABIHandler<'a, 'b> {
    /// Read argument
    pub fn read_arg<M>(&self, arg_offset: i32) -> Result<M, WasmV1Error>
    where
        M: proto::Message + Default,
    {
        let byte_vec = self
            .exec_env
            .read_buffer(&self.store_env, arg_offset)
            .map_err(|err| {
                WasmV1Error::RuntimeError(format!("Could not read ABI argument: {}", err))
            })?;
        M::decode(&mut Cursor::new(&byte_vec)).map_err(|err| {
            WasmV1Error::RuntimeError(format!("Could not deserialize ABI argument: {}", err))
        })
    }

    /// Return a value
    pub fn return_value<M>(&mut self, value: M) -> Result<i32, WasmV1Error>
    where
        M: proto::Message,
    {
        let mut buf = Vec::with_capacity(value.encoded_len());
        value.encode(&mut buf).map_err(|err| {
            WasmV1Error::RuntimeError(format!("Could not serialize ABI return value: {}", err))
        })?;
        self.exec_env
            .write_buffer(&mut self.store_env, &buf)
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

    /// Get gas costs
    pub fn get_gas_costs(&self) -> &GasCosts {
        self.exec_env.get_gas_costs()
    }

    /// Get gas cost
    pub fn get_gas_cost(&self, abi_name: &str) -> u64 {
        *self
            .exec_env
            .get_gas_costs()
            .abi_costs
            .get(abi_name)
            .unwrap_or(&0)
    }
}
