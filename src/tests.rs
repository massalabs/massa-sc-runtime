use crate::execution_impl::run;
use crate::settings::METERING;
use crate::types::Address;
use crate::types::Bytecode;
use crate::types::Interface;
use anyhow::bail;
use std::sync::Mutex;

pub type Ledger = std::collections::BTreeMap<Address, Bytecode>; // Byttecode instead of String

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
        print: |message| {
            println!("{}", message);
            MEM.lock()
                .unwrap()
                .insert("output".to_string(), message.as_bytes().to_vec());
            Ok(())
        },
        ..Default::default()
    }
}

#[test]
fn test_caller() {
    let interface = &new();
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/build/get_string.wat"
    ));
    let update_module: fn(address: &Address, module: &Bytecode) -> anyhow::Result<()> =
        interface.update_module;
    update_module(&"get_string".to_string(), &module.to_vec()).unwrap();
    run(module, 100, interface).expect("Failed to run get_string.wat");
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/build/caller.wat"
    ));
    let a = run(module, 20_000, interface).expect("Failed to run caller.wat");

    let prev_call_price = METERING.call_price();
    METERING._reset(0);
    let b = run(module, 20_000, interface).expect("Failed to run caller.wat");
    assert_eq!(a + prev_call_price, b);
    let mem = MEM.lock().unwrap();
    let output = std::str::from_utf8(mem.get("output").unwrap()).unwrap();
    assert_eq!(output, "hello you");
}

#[test]
#[should_panic]
fn test_local_hello_name_caller() {
    let interface = &new();
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/build/get_string.wat"
    ));
    let update_module: fn(address: &Address, module: &Bytecode) -> anyhow::Result<()> =
        interface.update_module;
    update_module(&"get_string".to_string(), &module.to_vec()).unwrap();
    run(module, 100, interface).expect("Failed to run get_string.wat");
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/build/local_hello_name_caller.wat"
    ));
    run(module, 20_000, interface).expect_err("Succeeded to run local_hello_name_caller.wat");
}
