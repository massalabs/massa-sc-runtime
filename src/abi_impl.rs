use crate::env::{get_remaining_points_for_env, sub_remaining_point, Env};
use crate::settings;
use crate::types::{Address, Response};
use anyhow::{Result, bail};
use as_ffi_bindings::{Read as ASRead, StringPtr, Write as ASWrite};

#[derive(Debug, Clone)]
pub(crate) struct ExitCode(pub(crate) String);
impl std::fmt::Display for ExitCode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for ExitCode {}
macro_rules! abi_bail {
    ($err:expr) => {
        wasmer::RuntimeError::raise(Box::new(crate::abi_impl::ExitCode($err.to_string())))
    };
}
pub(crate) use abi_bail; 

/// `Call` ABI called by the webassembly VM
///
/// Call an exported function in a WASM module at a given address
///
/// It take in argument the environment defined in env.rs
/// this environment is automatically filled by the wasmer library
/// And two pointers of string. (look at the readme in the wasm folder)
fn call_module(env: &Env, address: &Address, function: &str, param: &str) -> Result<Response> {
    if sub_remaining_point(env, settings::metering_call()).is_err() {
        bail!("Cannot substract remaining points")
    };
    if let Ok(module) = &env.interface.get_module(address) {
        crate::execution_impl::exec(
            get_remaining_points_for_env(env),
            None,
            module,
            function,
            param,
            &*env.interface,
        )
    } else {
        bail!("Cannot reach the target module at the address {}", address)
    }
}

/// Raw call that have the right type signature to be able to be call a module
/// directly form AssemblyScript:
///
#[doc = include_str!("../wasm/README.md")]
pub(crate) fn assembly_script_call_module(
    env: &Env,
    address: i32,
    function: i32,
    param: i32,
) -> i32 {
    let memory = env.wasm_env.memory.get_ref().expect("uninitialized memory");
    let addr_ptr = StringPtr::new(address as u32);
    let func_ptr = StringPtr::new(function as u32);
    let param_ptr = StringPtr::new(param as u32);

    let address = addr_ptr.read(memory);
    let function = func_ptr.read(memory);
    let param = param_ptr.read(memory);
    if address.is_err() || function.is_err() || param.is_err() {
        abi_bail!("Cannot read address, function or param in memory in call module request ABI")
    }
    let address = &address.unwrap();
    let function = &function.unwrap();
    let param = &param.unwrap();
    let value = call_module(env, address, function, param);
    if value.is_err() {
        abi_bail!(value.err().unwrap())
    }
    if let Ok(ret) = StringPtr::alloc(&value.unwrap().ret, &env.wasm_env) {
        ret.offset() as i32
    } else {
        abi_bail!(format!("Cannot allocate response in call {}::{}", address, function))
    }
}

pub(crate) fn get_remaining_points(env: &Env) -> i32 {
    if sub_remaining_point(env, settings::metering_call()).is_err() {
        abi_bail!("Cannot substract remaining points")
    };
    get_remaining_points_for_env(env) as i32
}

/// Create an instance of VM from a module with a
/// given intefrace, an operation number limit and a webassembly module
///
/// An utility print function to write on stdout directly from AssemblyScript:
pub(crate) fn assembly_script_print(env: &Env, arg: i32) {
    let str_ptr = StringPtr::new(arg as u32);
    let memory = env.wasm_env.memory.get_ref().expect("uninitialized memory");
    if let Ok(message) = &str_ptr.read(memory) {
        if env.interface.print(message).is_err() {
            abi_bail!("Failed to print message");
        }
    } else {
        abi_bail!("Cannot read message pointer in memory");
    }
}
