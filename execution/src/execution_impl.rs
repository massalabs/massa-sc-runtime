use crate::api;
use crate::env::{abort, get_remaining_points, Env};
use crate::types::{Address, Response};
use anyhow::{bail, Result};
use std::sync::Arc;
use wasmer::wasmparser::Operator;
use wasmer::{
    imports, CompilerConfig, Cranelift, Function, ImportObject, Instance, Module, Store, Universal,
    Val,
};
use wasmer_as::{Read as ASRead, StringPtr, Write as ASWrite};
use wasmer_middlewares::Metering;

const MAIN: &str = "main";

fn print(env: &Env, arg: i32) {
    let str_ptr = StringPtr::new(arg as u32);
    // May be in wasmer-as create a pointer that keep memory?
    println!(
        "\n{}",
        str_ptr
            .read(env.wasm_env.memory.get_ref().expect("initialized memory"))
            .unwrap()
    );
}

fn call(env: &Env, address: i32, function: i32) -> i32 {
    let memory = env.wasm_env.memory.get_ref().expect("initialized memory");
    let addr_ptr = StringPtr::new(address as u32);
    let func_ptr = StringPtr::new(function as u32);

    let address = &addr_ptr.read(memory).unwrap();
    let fnc = &func_ptr.read(memory).unwrap();
    let module = &api::get_module(address).unwrap();
    let value =
        super::execution_impl::exec(env.remaining_points, None, module, fnc, &vec![]).unwrap();
    let ret = StringPtr::alloc(&value.ret, &env.wasm_env).unwrap();
    ret.offset() as i32
}

fn create_instance(limit: u64, module: &[u8]) -> Result<Instance> {
    let metering = Arc::new(Metering::new(limit, |_: &Operator| -> u64 { 1 }));
    let mut compiler_config = Cranelift::default();
    compiler_config.push_middleware(metering);
    let store = Store::new(&Universal::new(compiler_config).engine());
    let resolver: ImportObject = imports! {
        "env" => {
            "abort" =>  Function::new_native_with_env(&store, Env::default(), abort)
        },
        "index" => {
            "print" => Function::new_native_with_env(&store, Env::default(), print),
            "call" => Function::new_native_with_env(&store, Env::default(), call),
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
) -> Result<Response> {
    let instance = match instance {
        Some(instance) => instance,
        None => create_instance(limit, module)?,
    };
    let mut p = vec![];
    for param in params {
        p.push(Val::I32(*param));
    }
    // todo: return an error if the function exported isn't public?
    match instance.exports.get_function(fnc)?.call(&p) {
        Ok(value) => {
            // todo: clean and define wat should be return by the main
            if fnc.eq(MAIN) {
                return Ok(Response {
                    ret: "0".to_string(),
                    remaining_points: 0,
                });
            }
            let str_ptr = StringPtr::new(value.get(0).unwrap().i32().unwrap() as u32);
            let memory = instance.exports.get_memory("memory")?;
            Ok(Response {
                ret: str_ptr.read(memory)?,
                remaining_points: get_remaining_points(&instance),
            })
        }
        Err(error) => bail!(error),
    }
}

pub fn run(address: Address, module: &[u8], limit: u64) -> Result<()> {
    let instance = create_instance(limit, module)?;
    // todo: what to export?
    println!(
        "Module inserted at {} by {}",
        api::insert_module(address.clone(), module),
        address
    );
    if instance.exports.contains(MAIN) {
        return match exec(limit, Some(instance), module, MAIN, &[]) {
            Ok(_) => Ok(()),
            Err(error) => bail!(error),
        };
    }
    Ok(())
}
