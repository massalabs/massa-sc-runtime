use wasmer::{Function, ImportObject, Memory, MemoryType, Instance, Module, Store, Val, imports, Type, FunctionType, MemoryView};
use crate::api;
use crate::types::Address;
use anyhow::{bail, Result};
use wasmer_as::{Env, abort, AsmScriptStringPtr, AsmScriptRead};
use std::{cell::Cell, io::{Bytes, Read}};
/*
index.tx

// The entry file of your WebAssembly module.

export declare function print(arg: string): void;
export declare function concat_world(arg: string): string;

export function main(): void {
  //console.log(call("hello"))
  let hello = "hello";
  hello = concat_world(hello);
  print(hello);
}

*/

fn create_instance(module: &[u8]) -> Result<Instance> {
    let store = Store::default();
    let memory = Memory::new(&store, MemoryType::new(1, None, false)).unwrap();
    memory.size();
    fn print(env: &Env, arg: i32) {
        let str_ptr = AsmScriptStringPtr::new(arg as u32);
        let memory = env.memory.get_ref().expect("initialized memory");
        let str_val = str_ptr.read(memory).unwrap();
        println!("Mem size: {}", memory.data_size());
        println!(">>> {}", str_val);
    }

    fn concat_world(env: &Env, arg: i32) -> i32 {
        let str_ptr = AsmScriptStringPtr::new(arg as u32);
        let mut memory = env.memory.get_ref().expect("initialized memory");
        let mut str_val = str_ptr.read(memory).unwrap();
        println!("offset: {}, header: {}", arg, arg / 4 - 1);
        println!("val red {}", str_val);
        let view: MemoryView<u32> = memory.view();

        let world = "world".bytes();
        let mut inject = vec![];
        for byte in world {
            inject.push(byte);
            inject.push(0);
        }
        let view8: MemoryView<u8> = memory.view();
        for (byte, cell) in inject.bytes().zip(view8[1000 as usize..(1000 + inject.len()) as usize].iter()) {
            cell.set(byte.unwrap());
        }
        let c = view.get(1000 as usize / 4 - 1).unwrap();
        c.set(10);
        return 1000;
    }

    let resolver: ImportObject = imports! {
        "env" => {
            "abort" =>  Function::new_native_with_env(&store, Env::default(), abort)
        },
        "index" => {
            "print" => Function::new_native_with_env(&store, Env::default(), print),
            "concat_world" => Function::new_native_with_env(&store, Env::default(), concat_world),
            
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