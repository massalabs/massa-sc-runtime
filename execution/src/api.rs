use crate::types::Address;
use super::types::{Bytecode, Ledger};
use std::sync::Mutex;
use anyhow::{bail, Result};

lazy_static::lazy_static! {
   pub static ref MEM: Mutex::<Ledger> = Mutex::new(Ledger::new());
}

pub fn get_module(address: &Address) -> Result<Bytecode> {
   match MEM.lock().unwrap().clone().get(address) {
      Some(module) => Ok(module.clone()),
      _ => bail!("Cannot find module for address {}", address)
   }
}

// Exporting a function not named "main" in a module result to store
// the module in the ledger
// Adding a module to execute in his own address
pub fn insert_module(address: Address, module: &[u8]) -> Address {
   let key = address + 1; // todo insert the logic of address creation
   MEM.lock().unwrap().insert(key, module.to_vec());
   key
}

//pub fn call(input: &[u64]) -> Result<Vec::<u64>> {
//   let module = get_module(&address).unwrap();
//   super::execution_impl::exec(None, &module, "", vec![]).unwrap()[0].to_string(); // TODO change exec
//   vec![]
//}