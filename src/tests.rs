use crate::run;
use crate::settings::METERING;
use crate::types::{Address, Bytecode, Ledger};
use crate::types::{Interface, InterfaceClone};
use crate::update_and_run;
use anyhow::{bail, Result};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct TestInterface(Arc<Mutex<Ledger>>);

impl InterfaceClone for TestInterface {
    fn clone_box(&self) -> Box<dyn Interface> {
        Box::new(self.clone())
    }
}

impl Interface for TestInterface {
    fn get_module(&self, address: &Address) -> Result<Bytecode> {
        match self.0.lock().unwrap().clone().get(&address.clone()) {
            Some(module) => Ok(module.clone()),
            _ => bail!("Cannot find module for address {}", address),
        }
    }

    fn update_module(&self, address: &Address, module: &Bytecode) -> Result<()> {
        self.0
            .lock()
            .unwrap()
            .insert(address.clone(), module.to_vec());
        Ok(())
    }
}

#[test]
fn test_caller() {
    let interface: Box<dyn Interface> =
        Box::new(TestInterface(Arc::new(Mutex::new(Ledger::new()))));
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/build/get_string.wat"
    ));
    update_and_run("get_string.wat".to_string(), module, 100, &interface)
        .expect("Failed to run get_string.wat");
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/build/caller.wat"
    ));
    let a = run(module, 20_000, &interface).expect("Failed to run caller.wat");
    let prev_call_price = METERING.call_price();
    METERING._reset(0);
    let b = run(module, 20_000, &interface).expect("Failed to run caller.wat");
    assert_eq!(a + prev_call_price, b);
}
