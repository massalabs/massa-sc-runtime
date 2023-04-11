//! *abi_impl.rs* contains all the implementation (and some tools as
//! abi_bail!) of the massa abi.
//!
//! The ABIs are the imported function / object declared in the webassembly
//! module. You can look at the other side of the mirror in `massa.ts` and the
//! rust side in `execution_impl.rs`.

use crate::Interface;

use super::{
    env::{ABIEnv, ExecutionEnv},
    WasmV1Error,
};
use wasmer::FunctionEnvMut;

/// Handle an ABI call by providing helpers to read arguments, return values, and so on
fn handle_abi<F, R>(mut store_env: FunctionEnvMut<ABIEnv>, func: F) -> Result<R, WasmV1Error>
where
    F: FnOnce(&mut ABIHandler) -> Result<R, WasmV1Error>,
{
    let env_mutex = store_env.data().clone();
    let env_lock = env_mutex.lock();
    let exec_env = env_lock.as_ref().ok_or_else(|| {
        WasmV1Error::InstanciationError("ABIs cannot be called at initialization time.".into())
    })?;
    let interface = &**exec_env.get_interface();

    let mut handler = ABIHandler {
        store_env: &mut store_env,
        exec_env,
        interface,
    };

    func(&mut handler)
}      

struct ABIHandler<'a, 'b> {
    store_env: &'b mut FunctionEnvMut<'a, ABIEnv>,
    exec_env: &'b ExecutionEnv,
    pub interface: &'b dyn Interface,
}

impl<'a, 'b> ABIHandler<'a, 'b> {
    /// Read argument
    pub fn read_arg(&self, arg_offset: i32) -> Result<Vec<u8>, WasmV1Error> {
        self.exec_env
            .read_buffer(&self.store_env, arg_offset)
            .map_err(|err| {
                WasmV1Error::RuntimeError(format!("Could not read ABI argument: {}", err))
            })
    }

    /// Return a value
    pub fn return_value(&mut self, value: &[u8]) -> Result<i32, WasmV1Error> {
        self.exec_env
            .write_buffer(&mut self.store_env, value)
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
}

/// Transfer an amount from the address on the current call stack to a target address.
pub(crate) fn abi_transfer_coins(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(store_env, |handler| {
        // Update gas
        handler.try_subtract_gas(10)?;

        // read params
        let _params = handler.read_arg(arg_offset)?;

        // execute
        //let result = handler.interface.transfer_coins(params)?;

        // write result
        handler.return_value(&vec![0u8, 1u8, 2u8, 3u8])
    })
}

/// Raw call that have the right type signature to be able to be call a module
/// directly form AssemblyScript:
pub(crate) fn abi_call(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(store_env, |handler| {
        // Update gas
        handler.try_subtract_gas(10)?;

        // read params
        let _params = handler.read_arg(arg_offset)?;

        // execute
        //let result = handler.interface.transfer_coins(params)?;

        // write result
        handler.return_value(&vec![0u8, 1u8, 2u8, 3u8])
    })
}

/// sets a key-indexed data entry in the datastore, overwriting existing values if any
pub(crate) fn abi_set_data(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(store_env, |handler| {
        // Update gas
        handler.try_subtract_gas(10)?;

        // read params
        let _params: SetDataRequest = handler.read_arg<SetDataRequest>(arg_offset)?;

        // execute
        //let result = handler.interface.transfer_coins(params)?;

        // write result
        handler.return_value(SetDataReponse)
    })
}

/// gets a key-indexed data entry in the datastore, failing if non-existent
pub(crate) fn abi_get_data(
    store_env: FunctionEnvMut<ABIEnv>,
    arg_offset: i32,
) -> Result<i32, WasmV1Error> {
    handle_abi(store_env, |handler| {
        // Update gas
        handler.try_subtract_gas(10)?;

        // read params
        let _params = handler.read_arg(arg_offset)?;

        // execute
        //let result = handler.interface.transfer_coins(params)?;

        // write result
        handler.return_value(&vec![0u8, 1u8, 2u8, 3u8])
    })
}

/// Function designed to abort execution.
pub fn abi_abort(store_env: FunctionEnvMut<ABIEnv>, arg_offset: i32) -> Result<(), WasmV1Error> {
    handle_abi(store_env, |handler| {
        // Update gas
        handler.try_subtract_gas(10)?;

        // read params
        let _params = handler.read_arg(arg_offset)?;

        // bail
        Err(WasmV1Error::RuntimeError(format!(
            "Execution aborted: {}",
            "TODO"
        )))
    })
}
