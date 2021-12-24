use crate::env::{get_remaining_points_for_env, sub_remaining_point, Env};
use crate::settings;
use crate::types::{Address, Response};
use anyhow::Result;
use wasmer_as::{Read as ASRead, StringPtr, Write as ASWrite};

/// `Call` ABI called by the webassembly VM
///
/// Call an exported function in a WASM module at a given address
///
/// It take in argument the environment defined in env.rs
/// this environment is automatically filled by the wasmer library
/// And two pointers of string. (look at the readme in the wasm folder)
fn call_module(env: &Env, address: &Address, function: &str, param: &str) -> Result<Response> {
    sub_remaining_point(env, settings::metering_call()).unwrap();
    let module = &env.interface.get_module(address).unwrap();
    crate::execution_impl::exec(
        get_remaining_points_for_env(env),
        None,
        module,
        function,
        param,
        &*env.interface,
    )
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
    // TODO: replace all unwrap() by expect()
    let addr_ptr = StringPtr::new(address as u32);
    let func_ptr = StringPtr::new(function as u32);
    let address = &addr_ptr.read(memory).unwrap();
    let function = &func_ptr.read(memory).unwrap();
    let p = StringPtr::new(param as u32);
    let value = call_module(env, address, function, &p.read(memory).unwrap())
        .expect("could not call module in assembly_script_call_module");
    let ret = StringPtr::alloc(&value.ret, &env.wasm_env).unwrap();
    ret.offset() as i32
}

pub(crate) fn get_remaining_points(env: &Env) -> i32 {
    sub_remaining_point(env, settings::metering_call())
        .expect("could not sub remaining points in how many");
    get_remaining_points_for_env(env) as i32
}

/// Create an instance of VM from a module with a
/// given intefrace, an operation number limit and a webassembly module
///
/// An utility print function to write on stdout directly from AssemblyScript:
pub(crate) fn assembly_script_print(env: &Env, arg: i32) {
    let str_ptr = StringPtr::new(arg as u32);
    env.interface
        .print(
            &str_ptr
                .read(env.wasm_env.memory.get_ref().expect("uninitialized memory"))
                .unwrap(),
        )
        .unwrap();
}
