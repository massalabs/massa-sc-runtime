use wasmer::{Function, ImportObject, Instance, Module, Store, Val, imports};
use crate::api;
use crate::types::Address;
use anyhow::{bail, Result};
use api::call;

fn create_instance(module: &str) -> Result<Instance> {
    let store = Store::default();
    let resolver: ImportObject = imports! {
        "massa" => {
            "call" => Function::new_native(&store, call)
        },
    };
    let module = Module::new(&store, &module)?;
    Ok(Instance::new(&module, &resolver)?)
}

/// fnc: function name
/// params: function arguments
pub fn exec(instance: Option<Instance>, module: &str, fnc: &str, params: Vec<Val>) -> Result<Box<[Val]>> {
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
pub fn run(address: Address, module: &str) -> Result<()> {
    let instance = create_instance(module)?;
    println!("Module inserted at {} by {}", api::insert_module(address, module), address);
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
// Mock the blockchain behavior to be able to simulate SC wasm chunck without dealing with the blockchain