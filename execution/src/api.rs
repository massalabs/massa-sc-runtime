use wasmer::{RuntimeError, Val};
use super::types::Address;
use std::string::String;
use std::collections::BTreeMap;

lazy_static::lazy_static! {
   static ref MEM: BTreeMap<Address, String> = BTreeMap::new();
}

pub fn call(args: &[Val]) -> Result::<Vec<Val>, RuntimeError> {
   let address = args[0].i64().unwrap() as u64; //todo : remove this cast
   match MEM.contains_key(&address) {
      true => {
         match super::run(&MEM.get(&address).unwrap(), &args[1].to_string(), vec![]) {
            Ok(_) => Ok(vec![]),
            Err(_) => Err(RuntimeError::new("Run call error"))
         }
      },
      false => Err(RuntimeError::new("Address not found"))
   }
}
