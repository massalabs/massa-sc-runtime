use wasmer::{Function, FunctionType, ImportObject, Instance, Module, RuntimeError, Store, Type, Val, imports};
use crate::api;
use crate::types::Address;

fn create_instance(module: &str) -> Result<Instance, Box<dyn std::error::Error>> {
    let store = Store::default();
    let resolver: ImportObject = imports! {
        "env" => {
            "call" => Function::new(&store, &FunctionType::new(vec![Type::I32], vec![Type::I32]), api::call),
        },
    };
    let module = Module::new(&store, &module)?;
    Ok(Instance::new(&module, &resolver)?)
}

pub fn exec(address: Address, instance: Option<Instance>, module: &str, fnc: &str, params: Vec<Val>) -> Result<Box<[Val]>, Box<dyn std::error::Error>> {
    let instance = match instance {
        Some(instance) => instance,
        None => create_instance(module)?
    };
    match instance.exports.get_function(fnc)?.call(&params) {
        Ok(value) => Ok(value),
        Err(error) => Err(Box::new(std::io::Error::new::<RuntimeError>(std::io::ErrorKind::InvalidData, error)))
    }
}

/// External access to run a module sended by an address
/// Is run could be split write_sc_to_ledger -> Address then exec(Address, function) ?!
///
/// - do we want to be able to execute a sSCc without save it to the ledger? -> is it still a SC how do we name it?
/// - do we want to save a SC to the ledger without execute it?
pub fn run(address: Address, module: &str) -> Result<(), Box<dyn std::error::Error>> {
    let instance = create_instance(module)?;
    // Insert module in the ledger if another function
    // than "main" is exported

    // todo: load is_pub instead of this bellow to store in the ledger
    for exp in instance.exports.iter() {
        if !exp.0.eq("main") {
            let scaddress = api::insert_module(address, module);
            break;
        }
    }

    if instance.exports.contains("main") {
        return match exec() {
            Ok(value) => Ok(()),
            Err(error) => Err(Box::new(std::io::Error::new::<RuntimeError>(std::io::ErrorKind::InvalidData, error)))
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