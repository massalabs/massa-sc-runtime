use wasmer::{Function, ImportObject, Instance, Module, Store, Val, imports};
use crate::api;
use crate::types::Address;
use anyhow::{bail, Result};
use wasmer_as::{Env, abort, StringPtr, Read as ASRead, Write as ASWrite};

const MAIN: &str = "main";

fn print(env: &Env, arg: i32) {
    let str_ptr = StringPtr::new(arg as u32);
    // May be in wasmer-as create a pointer that keep memory?
    println!("\n{}", str_ptr.read(env.memory.get_ref().expect("initialized memory")).unwrap());
}

fn call(env: &Env, address: i32, function: i32) -> i32 {
    let memory = env.memory.get_ref().expect("initialized memory");
    let addr_ptr = StringPtr::new(address as u32);
    let func_ptr = StringPtr::new(function as u32);

    let address = &addr_ptr.read(memory).unwrap();
    let fnc = &func_ptr.read(memory).unwrap();
    let module = &api::get_module(address).unwrap();

    let value = super::execution_impl::exec(None, module, fnc, &vec![]).unwrap();
    let ret = StringPtr::alloc(&value, env).unwrap();
    ret.offset() as i32
}

fn create_instance(module: &[u8]) -> Result<Instance> {
    let store = Store::default();
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
pub fn exec(instance: Option<Instance>, module: &[u8], fnc: &str, params: &[i32]) -> Result<String> {
    let instance = match instance {
        Some(instance) => instance,
        None => create_instance(module)?
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
                return Ok("0".to_string());
            }
            let str_ptr = StringPtr::new(value.get(0).unwrap().i32().unwrap() as u32);
            let memory = instance.exports.get_memory("memory")?;
            Ok(str_ptr.read(memory)?)
        },
        Err(error) => bail!(error)
    }
}

pub fn run(address: Address, module: &[u8]) -> Result<()> {
    let instance = create_instance(module)?;
    // todo: what to export?
    println!("Module inserted at {} by {}",
        api::insert_module(address.clone(), module), address);
    if instance.exports.contains(MAIN) {
        return match exec(Some(instance), module, MAIN, &[]) {
            Ok(_) => {
                Ok(())
            },
            Err(error) => bail!(error)
        }
    }
    Ok(())
}