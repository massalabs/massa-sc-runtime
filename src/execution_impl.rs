use crate::env::{
    assembly_script_abort, get_remaining_points, get_remaining_points_instance,
    sub_remaining_point, Env,
};
use crate::settings;
use crate::types::{Address, Bytecode, Interface, Response};
use anyhow::{bail, Result};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::sync::Arc;
use wasmer::wasmparser::Operator;
use wasmer::{
    imports, CompilerConfig, Cranelift, Function, ImportObject, Instance, Module, Store, Universal,
    Val,
};
use wasmer_as::{Read as ASRead, StringPtr, Write as ASWrite};
use wasmer_middlewares::Metering;

/// Utility function to call a WASM module (compiled from AssemblyScript) using
/// high-level Rust types (by de/serializing a String).
///
/// The intend is to be able to expose any kind of function as an ABI:
///
/// ```ignore
/// fn get_status() -> Result<NodeStatus> {
///     Ok(typed_call(env, "0xAddressOfSCInLedger", "get_status", ())?)
/// }
/// ```
pub fn _typed_call<T: Serialize, R: DeserializeOwned>(
    _env: &Env,
    _address: &Address,
    _function: &str,
    _args: T, // TODO: @adrien-zinger how do we pass function args in AssemblyScript?
) -> Result<R> {
    //    let value = call_module(env, address, function, _args);
    //    Ok(serde_json::from_str::<R>(&value.ret)?)
    todo!()
}

/// `Call` ABI called by the webassembly VM
///
/// Call an exported function in a WASM module at a given address
///
/// It take in argument the environment defined in env.rs
/// this environment is automatically filled by the wasmer library
/// And two pointers of string. (look at the readme in the wasm folder)
fn call_module(env: &Env, address: &Address, function: &str, param: &str) -> Response {
    let get_module: fn(&Address) -> Result<Bytecode> = env.interface.get_module;
    sub_remaining_point(env, settings::METERING.call_price()).unwrap();
    let module = &get_module(address).unwrap();
    exec(
        get_remaining_points(env),
        None,
        module,
        function,
        param,
        &env.interface,
    )
    .unwrap() // TODO: Should return a Result<Response>
}

/// Raw call that have the right type signature to be able to be call a module
/// directly form AssemblyScript:
///
#[doc = include_str!("../wasm/README.md")]
fn assembly_script_call_module(env: &Env, address: i32, function: i32, param: i32) -> i32 {
    let memory = env.wasm_env.memory.get_ref().expect("initialized memory");
    // TODO: replace all unwrap() by expect()
    let addr_ptr = StringPtr::new(address as u32);
    let func_ptr = StringPtr::new(function as u32);
    let address = &addr_ptr.read(memory).unwrap();
    let function = &func_ptr.read(memory).unwrap();
    let p = StringPtr::new(param as u32);
    let value = call_module(env, address, function, &p.read(memory).unwrap());
    let ret = StringPtr::alloc(&value.ret, &env.wasm_env).unwrap();
    ret.offset() as i32
}

fn how_many(env: &Env) -> i32 {
    sub_remaining_point(env, 15).expect("could not sub remaining points in how many");
    get_remaining_points(env) as i32
}

/// Create an instance of VM from a module with a
/// given intefrace, an operation number limit and a webassembly module
///
/// An utility print function to write on stdout directly from AssemblyScript:
fn assembly_script_print(env: &Env, arg: i32) {
    let str_ptr = StringPtr::new(arg as u32);
    let print: fn(&str) -> Result<()> = env.interface.print;
    print(
        &str_ptr
            .read(env.wasm_env.memory.get_ref().expect("uninitialized memory"))
            .unwrap(),
    )
    .unwrap();
}

/// Create an instance of VM from a module with a given interface, an operation
/// number limit and a webassembly module
fn create_instance(limit: u64, module: &[u8], interface: &Interface) -> Result<Instance> {
    let metering = Arc::new(Metering::new(limit, |_: &Operator| -> u64 { 1 }));
    let mut compiler_config = Cranelift::default();
    compiler_config.push_middleware(metering);
    let store = Store::new(&Universal::new(compiler_config).engine());
    let env = Env::new(interface);
    let resolver: ImportObject = imports! {
        "env" => {
            "abort" =>  Function::new_native_with_env(&store, env.clone(), assembly_script_abort),
        },
        "massa" => {
            "assembly_script_print" => Function::new_native_with_env(&store, env.clone(), assembly_script_print),
            "assembly_script_call" => Function::new_native_with_env(&store, env.clone(), assembly_script_call_module),
            "how_many" => Function::new_native_with_env(&store, env, how_many),
        },
    };
    let module = Module::new(&store, &module)?;
    Ok(Instance::new(&module, &resolver)?)
}

fn exec(
    limit: u64,
    instance: Option<Instance>,
    module: &[u8],
    function: &str,
    param: &str,
    interface: &Interface,
) -> Result<Response> {
    let instance = match instance {
        Some(instance) => instance,
        None => create_instance(limit, module, interface)?,
    };
    // TODO find a way to get an env from instance, or to allocate from instance in wasmer-as.
    let memory = instance.exports.get_memory("memory").unwrap();
    let env = wasmer_as::Env::new(
        memory.clone(),
        match instance.exports.get_function("__new") {
            Ok(func) => Some(func.clone()),
            _ => None,
        },
    );
    let param_ptr = *StringPtr::alloc(param, &env)?;
    // todo: return an error if the function exported isn't public?
    match instance
        .exports
        .get_function(function)?
        .call(&[Val::I32(param_ptr.offset() as i32)])
    {
        Ok(value) => {
            // TODO: clean and define wat should be return by the main
            if function.eq(crate::settings::MAIN) {
                return Ok(Response {
                    ret: "0".to_string(),
                    remaining_points: get_remaining_points_instance(&instance),
                });
            }
            let str_ptr = StringPtr::new(value.get(0).unwrap().i32().unwrap() as u32);
            let memory = instance.exports.get_memory("memory")?;
            Ok(Response {
                ret: str_ptr.read(memory)?,
                remaining_points: get_remaining_points_instance(&instance),
            })
        }
        Err(error) => bail!(error),
    }
}

pub fn run(module: &[u8], limit: u64, interface: &Interface) -> Result<u64> {
    let instance = create_instance(limit, module, interface)?;
    if instance.exports.contains(settings::MAIN) {
        return match exec(limit, Some(instance), module, settings::MAIN, "", interface) {
            Ok(result) => Ok(result.remaining_points),
            Err(error) => bail!(error),
        };
    }
    Ok(limit)
}
