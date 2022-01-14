use crate::execution_impl::run;
use crate::settings;
use crate::types::{Address, Bytecode};
use crate::types::{Interface, InterfaceClone};
use anyhow::{bail, Result};
use serial_test::serial;
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

    fn get_balance(&self) -> Result<u64> {
        Ok(1)
    }

    fn get_balance_for(&self, _address: &Address) -> Result<u64> {
        Ok(1)
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

    fn get_data(&self, _: &String) -> Result<Bytecode> {
        match self.0.lock().unwrap().clone().get(&"print".to_string()) {
            Some(bytes) => Ok(bytes.clone()),
            _ => bail!("Cannot find data"),
        }
    }

    fn create_module(&self, module: &Bytecode) -> Result<Address> {
        let address = String::from("get_string");
        self.0
            .lock()
            .unwrap()
            .insert(address.clone(), module.clone());
        Ok(address)
    }

    fn exit_success(&self) -> Result<()> {
        Ok(())
    }
}

#[test]
#[serial]
fn test_caller() {
    settings::reset_metering();
    let interface: Box<dyn Interface> =
        Box::new(TestInterface(Arc::new(Mutex::new(Ledger::new()))));
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/build/get_string.wat"
    ));
    interface
        .update_module(&"get_string".to_string(), &module.to_vec())
        .unwrap();
    // test only if the module is valid
    run(module, 20_000, &*interface).expect("Failed to run get_string.wat");
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/build/caller.wat"
    ));
    let a = run(module, 20_000, &*interface).expect("Failed to run caller.wat");
    let prev_call_price = settings::metering_call();
    settings::set_metering(0);
    let b = run(module, 20_000, &*interface).expect("Failed to run caller.wat");
    assert_eq!(a + prev_call_price, b);
    let v_out = interface.get_data(&String::new()).unwrap();
    let output = std::str::from_utf8(&v_out).unwrap();
    assert_eq!(output, "hello you");

    // Test now if we failed if metering is too hight
    settings::set_metering(15_000);
    run(module, 20_000, &*interface).expect_err("Expected to be out of operation gas");
}

#[test]
#[serial]
fn test_get_balance() {
    settings::reset_metering();
    let interface: Box<dyn Interface> =
        Box::new(TestInterface(Arc::new(Mutex::new(Ledger::new()))));
    assert!(interface.get_balance().is_ok());
}

#[test]
#[serial]
fn test_get_balance_for() {
    settings::reset_metering();
    let interface: Box<dyn Interface> =
        Box::new(TestInterface(Arc::new(Mutex::new(Ledger::new()))));
    assert!(interface.get_balance_for(&"test".to_string()).is_ok());
}

#[test]
#[serial]
fn test_local_hello_name_caller() {
    settings::reset_metering();
    // This test should verify that even if we failed to load a module,
    // we should never panic and just stop the call stack
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

#[test]
#[serial]
fn test_module_creation() {
    settings::reset_metering();
    // This test should create a smartcontract module and call it
    let interface: Box<dyn Interface> =
        Box::new(TestInterface(Arc::new(Mutex::new(Ledger::new()))));
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/build/create_sc.wasm"
    ));
    run(module, 100_000, &*interface).expect("Failed to run create_sc.wat");
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/build/caller.wat"
    ));
    run(module, 20_000, &*interface).expect("Failed to run caller.wat");
}
