use wasmer::{RuntimeError, Val};
use crate::types::Address;
use super::types::{Bytecode, Ledger};
use std::sync::Mutex;
use std::io::{Error, ErrorKind};

lazy_static::lazy_static! {
   pub static ref MEM: Mutex::<Ledger> = Mutex::new(Ledger::new());
}

pub fn get_module(address: &Address) -> Result<Bytecode, Error>{
   match MEM.lock().unwrap().clone().get(address) {
      Some(address) => Ok(address.to_string()),
      _ => Err(Error::new(ErrorKind::InvalidData, format!("Cannot find module for address {}", address)))
   }
}

// Exporting a function not named "main" in a module result to store
// the module in the ledger
// Adding a module to execute in his own address
pub fn insert_module(address: Address, module_wat: &str) {
   MEM.lock().unwrap().insert(address, module_wat.to_string());
}

pub fn call(args: &[Val]) -> Result::<Vec<Val>, RuntimeError> {
   let address = args[0].i64().unwrap() as u64; //todo : remove this cast
   match get_module(&address) {
      Ok(module_wat) => {
         match (&module_wat, &args[1].to_string(), vec![]) {
            Ok(_) => Ok(vec![]), // todo
            Err(_) => Err(RuntimeError::new("Run call error")) // todo
         }
      },
      Err(err) => Err(RuntimeError::new(err.to_string()))
   }
}
