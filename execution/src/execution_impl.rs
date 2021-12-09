use crate::env::{abort, get_remaining_points, Env, sub_remaining_point, get_remaining_points_instance};
use crate::types::{Address, Bytecode, Interface, Response};
use crate::settings;
use anyhow::{bail, Result};
use std::sync::Arc;
use wasmer::wasmparser::Operator;
use wasmer::{
    imports, CompilerConfig, Cranelift, Function, ImportObject, Instance, Module, Store, Universal,
    Val,
};
use wasmer_as::{Read as ASRead, StringPtr, Write as ASWrite};
use wasmer_middlewares::Metering;

/// `Print` ABI called by the webassembly VM
fn print(env: &Env, arg: i32) {
    let str_ptr = StringPtr::new(arg as u32);
    println!("{}",
        str_ptr
            .read(env.wasm_env.memory.get_ref().expect("initialized memory"))
            .unwrap()
    );
}

/// `Call` ABI called by the webassembly VM
/// It take in argument the environment defined in env.rs
/// this environment is automatically filled by the wasmer library
/// And two pointers of string. (look at the readme in the wasm folder)
fn call(env: &Env, address: i32, function: i32) -> i32 {
    let memory = env.wasm_env.memory.get_ref().expect("initialized memory");
    let addr_ptr = StringPtr::new(address as u32);
    let func_ptr = StringPtr::new(function as u32);
    let address = &addr_ptr.read(memory).unwrap();
    let fnc = &func_ptr.read(memory).unwrap();
    type GmSign = fn(&Address) -> Result<Bytecode>;
    let get_module: GmSign = env.interface.get_module;
    let module = &get_module(address).unwrap();
    sub_remaining_point(env, settings::METERING.call_price()).unwrap();
    let value = exec(
        get_remaining_points(env),
        None,
        module,
        fnc,
        &[],
        &env.interface,
    )
    .unwrap();
    let ret = StringPtr::alloc(&value.ret, &env.wasm_env).unwrap();
    ret.offset() as i32
}

/// Create an instance of VM from a module with a
/// given intefrace, an operation number limit and a webassembly module
/// 
fn create_instance(limit: u64, module: &[u8], interface: &Interface) -> Result<Instance> {
    let metering = Arc::new(Metering::new(limit, |_: &Operator| -> u64 { 1 }));
    let mut compiler_config = Cranelift::default();
    compiler_config.push_middleware(metering);
    let store = Store::new(&Universal::new(compiler_config).engine());
    let resolver: ImportObject = imports! {
        "env" => {
            "abort" =>  Function::new_native_with_env(&store, Env::new(interface), abort)
        },
        "index" => {
            "print" => Function::new_native_with_env(&store, Env::new(interface), print),
            "call" => Function::new_native_with_env(&store, Env::new(interface), call),
        },
    };
    let module = Module::new(&store, &module)?;
    Ok(Instance::new(&module, &resolver)?)
}

/// fnc: function name
/// params: function arguments
pub fn exec(
    limit: u64,
    instance: Option<Instance>,
    module: &[u8],
    fnc: &str,
    params: &[i32],
    interface: &Interface,
) -> Result<Response> {
    let instance = match instance {
        Some(instance) => instance,
        None => create_instance(limit, module, interface)?,
    };
    let mut p = vec![];
    for param in params {
        p.push(Val::I32(*param));
    }
    // todo: return an error if the function exported isn't public?
    match instance.exports.get_function(fnc)?.call(&p) {
        Ok(value) => {
            // todo: clean and define wat should be return by the main
            if fnc.eq(crate::settings::MAIN) {
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

pub fn update_and_run(address: Address, module: &[u8], limit: u64, interface: &Interface) -> Result<u64> {
    type UmSignature = fn(address: &Address, module: &Bytecode) -> Result<()>;
    let update_module: UmSignature = interface.update_module;
    update_module(&address, &module.to_vec())?;
    println!("Module inserted by {}", address);
    run(module, limit, interface)
}

pub fn run(module: &[u8], limit: u64, interface: &Interface) -> Result<u64> {
    let instance = create_instance(limit, module, interface)?;
    if instance.exports.contains(settings::MAIN) {
        return match exec(limit, Some(instance), module, settings::MAIN, &[], interface) {
            Ok(result) => Ok(result.remaining_points),
            Err(error) => bail!(error),
        };
    }
    Ok(limit)
}