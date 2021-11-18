use wasmer::{Function, FunctionType, Type, Instance, Module, Store, imports};
use crate::api;

lazy_static::lazy_static! {
    static ref STORE: Store = Store::default();
    //static ref CALL: Function = Function::new(&STORE, &FunctionType::new(vec![Type::I32], vec![Type::I32]), api::call);
}

fn instanciate(module_wat: &str) ->  Result<Instance, Box<dyn std::error::Error>> {
    let module = Module::new(&STORE, &module_wat)?;
    let resolver = imports! {
        "env" => {
            "call" => Function::new(&STORE, &FunctionType::new(vec![Type::I32], vec![Type::I32]), api::call),
        },
    };
    Ok(Instance::new(&module, &resolver)?)
}

pub fn run(module_wat: &str) -> Result<(), Box<dyn std::error::Error>> {
    let instance = instanciate(module_wat)?;
    for exp in instance.exports.iter() {
        println!("{}", exp.0);
    }
    todo!("Execute main function in instance");
}
