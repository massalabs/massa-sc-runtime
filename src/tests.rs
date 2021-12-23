use crate::execution_impl::run;
use crate::settings;
use crate::types::{Address, Bytecode};
use crate::types::{Interface, InterfaceClone};
use anyhow::{bail, Result};
use std::sync::{Arc, Mutex};

pub type Ledger = std::collections::BTreeMap<Address, Bytecode>; // Byttecode instead of String

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

    fn print(&self, message: &str) -> Result<()> {
        println!("{}", message);
        self.0
            .lock()
            .unwrap()
            .insert("print".to_string(), message.as_bytes().to_vec());
        Ok(())
    }

    fn get_data(&self, _: &Address, _: &str) -> Result<Bytecode> {
        match self.0.lock().unwrap().clone().get(&"print".to_string()) {
            Some(bytes) => Ok(bytes.clone()),
            _ => bail!("Cannot find data"),
        }
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
    interface
        .update_module(&"get_string".to_string(), &module.to_vec())
        .unwrap();
    run(module, 20_000, &*interface).expect("Failed to run get_string.wat");
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/build/caller.wat"
    ));
    let a = run(module, 20_000, &*interface).expect("Failed to run caller.wat");
    let prev_call_price = settings::call_price();
    settings::reset_metering(0);
    let b = run(module, 20_000, &*interface).expect("Failed to run caller.wat");
    assert_eq!(a + prev_call_price, b);
    // todo Test with set_data & get_data
    let v_out = interface.get_data(&String::new(), &String::new()).unwrap();
    let output = std::str::from_utf8(&v_out).unwrap();
    assert_eq!(output, "hello you");
}

#[test]
#[should_panic]
fn test_local_hello_name_caller() {
    let interface: Box<dyn Interface> =
        Box::new(TestInterface(Arc::new(Mutex::new(Ledger::new()))));
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/build/get_string.wat"
    ));
    interface
        .update_module(&"get_string".to_string(), &module.to_vec())
        .unwrap();
    run(module, 100, &*interface).expect("Failed to run get_string.wat");
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/build/local_hello_name_caller.wat"
    ));
    run(module, 20_000, &*interface).expect_err("Succeeded to run local_hello_name_caller.wat");
}
