use anyhow::bail;
use execution::types::{Interface, Ledger};
use std::sync::Mutex;

lazy_static::lazy_static! {
   pub static ref MEM: Mutex::<Ledger> = Mutex::new(Ledger::new());
}

pub fn new() -> Interface {
    Interface {
        get_module: |address| match MEM.lock().unwrap().clone().get(&address.clone()) {
            Some(module) => Ok(module.clone()),
            _ => bail!("Cannot find module for address {}", address),
        },
        update_module: |address, module| {
            MEM.lock().unwrap().insert(address.clone(), module.to_vec());
            Ok(())
        },
        ..Default::default()
    }
}
