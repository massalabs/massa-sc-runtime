use wasmer::{Function, ImportObject, Memory, MemoryType, MemoryView, Instance, Module, Store, Val, imports, Type, FunctionType, Value};
use crate::api;
use crate::types::Address;
use anyhow::{bail, Result};
use wasmer_as::{Env, abort};
use std::cell::Cell;
/*


index.tx

// The entry file of your WebAssembly module.

export declare function call(arg: string): string

export function add(a: i32, b: i32): i32 {
  return a + b;
}

export function main(): i32 {
  //console.log(call("hello"))
  let hello = "hello world";
  call(hello);
  return hello.length;
}


===> optimized.wasm or optimized.wat (same code)



*/
fn create_instance(module: &[u8]) -> Result<Instance> {
    let store = Store::default();
    let signature = FunctionType::new(vec![Type::I32], vec![Type::I32]);
    let memory = Memory::new(&store, MemoryType::new(1, None, true)).unwrap();
    let resolver: ImportObject = imports! {
        "env" => {
            "abort" =>  Function::new_native_with_env(&store, Env::default(), abort)
        },
        "index" => {
            "call" => Function::new(&store, signature, move |args: &[Value]| {
                // the call function is ran like this:
                // export function main(): i32 {
                //   let hello = "hello world";
                //   call(hello);
                //  return hello.length;
                //}
                //


                // Now print the hello world here with `println!(hello)`


                // I tried this... doesn't do anything
                //for arg in args {
                //    println!("arg: {}", arg.i32().unwrap());
                //}
                //let view: MemoryView<u8> = memory.view();
                //for byte in view[0x1056..0x1065].iter().map(Cell::get) {
                //    println!("byte: {}", byte);
                //}
                Ok(args.to_vec())
            }),
        },
    };
    let module = Module::new(&store, &module)?;
    Ok(Instance::new(&module, &resolver)?)
}

/// fnc: function name
/// params: function arguments
pub fn exec(instance: Option<Instance>, module: &[u8], fnc: &str, params: Vec<Val>) -> Result<Box<[Val]>> {

    let instance = match instance {
        Some(instance) => instance,
        None => create_instance(module)?
    };
    // todo: return an error if the function exported isn't public
    match instance.exports.get_function(fnc)?.call(&params) {
        Ok(value) => Ok(value),
        Err(error) => bail!(error)
    }
}

/// External access to run a module sended by an address
/// Is run could be split write_sc_to_ledger -> Address then exec(Address, function) ?!
///
/// - do we want to be able to execute a sSCc without save it to the ledger? -> is it still a SC how do we name it?
/// - do we want to save a SC to the ledger without execute it?
pub fn run(address: Address, module: &[u8]) -> Result<()> {
    let instance = create_instance(module)?;
    println!("Module inserted at {} by {}",
        api::insert_module(address, module), address);
    if instance.exports.contains("main") {
        return match exec(Some(instance), module, "main", vec![]) {
            Ok(value) => {
                println!("Main function dumped {:?}", value[0]);
                Ok(())
            },
            Err(error) => bail!(error)
        }
    }
    Ok(())
}

// --- What does the LEDGER look like? ---
// How do I know what's belong to me in the ledger User <-> [SC]

// --- What is lacking here before merging with @greg skelton / ask review from @damip? ---
// What subset of this code could be re-use as execution engine in massa-node? all the `execution` lib!

// --- Purpose of the main? ---
// Mock the blockchain behavior to be able to simulate SC wasm chunck without dealing with the blockchainmessage