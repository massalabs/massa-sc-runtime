use crate::types::Address;
use super::types::{Bytecode, Ledger};
use std::sync::Mutex;
use anyhow::{bail, Result};

lazy_static::lazy_static! {
   pub static ref MEM: Mutex::<Ledger> = Mutex::new(Ledger::new());
}

pub fn get_module(address: &Address) -> Result<Bytecode> {
   match MEM.lock().unwrap().clone().get(&address.clone()) {
      Some(module) => Ok(module.clone()),
      _ => bail!("Cannot find module for address {}", address)
   }
}

// Exporting a function not named "main" in a module result to store
// the module in the ledger
// Adding a module to execute in his own address
pub fn insert_module(address: Address, module: &[u8]) -> Address {
   MEM.lock().unwrap().insert(address.clone(), module.to_vec());
   address
}
