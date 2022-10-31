/// THIS FILE SHOULD TEST THE ABI, NOT THE MOCKED INTERFACE
use crate::{
    run_function, run_main, settings,
    types::{Interface, InterfaceClone},
};
use anyhow::{anyhow, bail, Result};
use parking_lot::Mutex;
use rand::Rng;
use serial_test::serial;
use std::collections::BTreeMap;
use std::sync::Arc;
pub type Ledger = std::collections::BTreeMap<String, Vec<u8>>; // Bytecode instead of String

#[derive(Clone)]
struct TestInterface(Arc<Mutex<Ledger>>);

impl InterfaceClone for TestInterface {
    fn clone_box(&self) -> Box<dyn Interface> {
        Box::new(self.clone())
    }
}

impl Interface for TestInterface {
    fn init_call(&self, address: &str, _raw_coins: u64) -> Result<Vec<u8>> {
        let data = self.0.lock().clone();
        match data.get::<String>(&address.to_string()) {
            Some(module) => Ok(module.clone()),
            _ => bail!("Cannot find module for address {}", address),
        }
    }

    fn finish_call(&self) -> Result<()> {
        Ok(())
    }

    fn get_balance(&self) -> Result<u64> {
        Ok(1)
    }

    fn get_balance_for(&self, _address: &str) -> Result<u64> {
        Ok(1)
    }

    fn raw_set_bytecode_for(&self, address: &str, bytecode: &[u8]) -> Result<()> {
        self.0.lock().insert(address.to_string(), bytecode.to_vec());
        Ok(())
    }

    fn raw_set_bytecode(&self, bytecode: &[u8]) -> Result<()> {
        let address = String::from("get_string");
        self.0.lock().insert(address, bytecode.to_vec());
        Ok(())
    }

    fn print(&self, message: &str) -> Result<()> {
        self.0
            .lock()
            .insert("print".into(), message.as_bytes().to_vec());
        Ok(())
    }

    fn raw_get_data(&self, _: &str) -> Result<Vec<u8>> {
        let bytes = self.0.lock().clone();
        match bytes.get(&"print".to_string()) {
            Some(bytes) => Ok(bytes.clone()),
            _ => bail!("Cannot find data"),
        }
    }

    fn get_call_coins(&self) -> Result<u64> {
        Ok(0)
    }

    fn create_module(&self, module: &[u8]) -> Result<String> {
        let address = String::from("get_string");
        self.0.lock().insert(address.clone(), module.to_vec());
        Ok(address)
    }

    fn send_message(
        &self,
        _target_address: &str,
        _target_handler: &str,
        _validity_start: (u64, u8),
        _validity_end: (u64, u8),
        _max_gas: u64,
        _gas_price: u64,
        _coins: u64,
        _data: &[u8],
    ) -> Result<()> {
        Ok(())
    }

    fn get_op_keys(&self) -> Result<Vec<Vec<u8>>> {
        Ok(vec![
            vec![0, 1, 2, 3, 4, 5, 6, 11],
            vec![127, 128],
            vec![254, 255],
        ])
    }

    fn has_op_key(&self, key: &[u8]) -> Result<bool> {
        let ds: BTreeMap<Vec<u8>, Vec<u8>> = BTreeMap::from([
            (vec![0, 1, 2, 3, 4, 5, 6, 11], vec![65]),
            (vec![127, 128], vec![66, 67]),
            (vec![254, 255], vec![68, 69]),
        ]);

        Ok(ds.contains_key(key))
    }

    fn get_op_data(&self, key: &[u8]) -> Result<Vec<u8>> {
        let ds: BTreeMap<Vec<u8>, Vec<u8>> = BTreeMap::from([
            (vec![0, 1, 2, 3, 4, 5, 6, 11], vec![65]),
            (vec![127, 128], vec![66, 67]),
            (vec![254, 255], vec![68, 69]),
        ]);

        ds.get(key)
            .cloned()
            .ok_or_else(|| anyhow!("Unknown key: {:?}", key))
    }

    fn unsafe_random_f64(&self) -> Result<f64> {
        let ret: f64 = rand::random();
        println!("use random: {ret}");
        Ok(ret)
    }

    fn get_time(&self) -> Result<u64> {
        Ok(1580515200000)
    }
}

#[test]
#[serial]
fn test_caller() {
    settings::reset_metering();
    let interface: Box<dyn Interface> =
        Box::new(TestInterface(Arc::new(Mutex::new(Ledger::new()))));
    let mut module = vec![1u8];
    module.extend_from_slice(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/build/get_string.wasm"
    )));
    interface
        .raw_set_bytecode_for("get_string", &module)
        .unwrap();
    // test only if the module is valid
    run_main(&module, 20_000, &*interface).expect("Failed to run_main get_string.wasm");
    let mut module = vec![1u8];
    module.extend_from_slice(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/build/caller.wasm"
    )));
    let a = run_main(&module, 20_000, &*interface).expect("Failed to run_main caller.wasm");
    let prev_call_price = settings::metering_call();
    settings::set_metering(0);
    let b = run_main(&module, 20_000, &*interface).expect("Failed to run_main caller.wasm");
    assert_eq!(a + prev_call_price, b);
    let v_out = interface.raw_get_data("").unwrap();
    let output = std::str::from_utf8(&v_out).unwrap();
    assert_eq!(output, "hello you");

    // Test now if we failed if metering is too high
    settings::set_metering(15_000);
    run_main(&module, 20_000, &*interface).expect_err("Expected to be out of operation gas");
}

#[test]
#[serial]
fn test_caller_no_return() {
    settings::reset_metering();
    let interface: Box<dyn Interface> =
        Box::new(TestInterface(Arc::new(Mutex::new(Ledger::new()))));
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/build/get_string.wasm"
    ));
    interface.create_module(module.as_ref()).unwrap();
    // test only if the module is valid
    run_main(module, 20_000, &*interface).expect("Failed to run get_string.wasm");
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/build/caller_no_return.wasm"
    ));
    run_main(module, 20_000, &*interface).expect("Failed to run caller.wasm");
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
        "/wasm/build/get_string.wasm"
    ));
    interface
        .raw_set_bytecode_for("get_string", module.as_ref())
        .unwrap();
    run_main(module, 100, &*interface).expect("Failed to run_main get_string.wasm");
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/build/local_hello_name_caller.wasm"
    ));
    run_main(module, 20_000, &*interface)
        .expect_err("Succeeded to run_main local_hello_name_caller.wasm");
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
    run_main(module, 100_000, &*interface).expect("Failed to run_main create_sc.wasm");
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/build/caller.wasm"
    ));
    run_main(module, 20_000, &*interface).expect("Failed to run_main caller.wasm");
}

#[test]
#[serial]
fn test_not_enough_gas_error() {
    settings::reset_metering();
    // This test should create a smartcontract module and call it
    let interface: Box<dyn Interface> =
        Box::new(TestInterface(Arc::new(Mutex::new(Ledger::new()))));
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/build/create_sc.wasm"
    ));
    run_main(module, 100_000, &*interface).expect("Failed to run_main create_sc.wasm");
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/build/caller.wasm"
    ));
    match run_main(module, 10000, &*interface) {
        Ok(_) => panic!("Shouldn't pass successfully =-("),
        Err(err) => {
            assert!(err
                .to_string()
                .starts_with("RuntimeError: Not enough gas, limit reached at:"))
        }
    }
}

#[test]
#[serial]
fn test_send_message() {
    settings::reset_metering();
    let interface: Box<dyn Interface> =
        Box::new(TestInterface(Arc::new(Mutex::new(Ledger::new()))));
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/build/send_message.wasm"
    ));
    run_main(module, 100_000, &*interface).expect("Failed to run_main send_message.wasm");
}

#[test]
#[serial]
fn test_run_function() {
    settings::reset_metering();
    let interface: Box<dyn Interface> =
        Box::new(TestInterface(Arc::new(Mutex::new(Ledger::new()))));
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/build/receive_message.wasm"
    ));
    run_function(module, 100_000, "receive", "data", &*interface)
        .expect("Failed to run_function receive_message.wasm");
}

#[test]
#[serial]
fn test_run_main_without_main() {
    settings::reset_metering();
    let interface: Box<dyn Interface> =
        Box::new(TestInterface(Arc::new(Mutex::new(Ledger::new()))));
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/build/no_main.wasm"
    ));
    run_main(module, 100_000, &*interface).expect_err("An error should spawn here");
}

#[test]
#[serial]
fn test_run_empty_main() {
    settings::reset_metering();
    let interface: Box<dyn Interface> =
        Box::new(TestInterface(Arc::new(Mutex::new(Ledger::new()))));
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/build/empty_main.wasm"
    ));
    // Even if our SC is empty; there is still an initial and minimum metering cost
    // (mainly because we have a memory allocator to init)
    settings::set_metering_initial_cost(0);
    let a = run_main(module, 10_000_000, &*interface).expect("Failed to run empty_main.wasm");
    // Here we avoid hard-coding a value (that can change in future wasmer release)$
    assert!(a > 0);

    let mut rng = rand::thread_rng();
    let cost = rng.gen_range(1..1_000_000);
    settings::set_metering_initial_cost(cost);
    let b = run_main(module, 10_000_000, &*interface).expect("Failed to run empty_main.wasm");
    // Between 2 calls, the metering cost should be the difference
    assert_eq!(a - b, cost);
}

#[test]
#[serial]
fn test_op_fn() {
    settings::reset_metering();
    let interface: Box<dyn Interface> =
        Box::new(TestInterface(Arc::new(Mutex::new(Ledger::new()))));
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/build/op_fn.wasm"
    ));
    run_main(module, 10_000_000, &*interface).expect("Failed to run op_fn.wasm");
}

/// Test seed, now, abort and trace
#[test]
#[serial]
fn test_builtins() {
    settings::reset_metering();
    let interface: Box<dyn Interface> =
        Box::new(TestInterface(Arc::new(Mutex::new(Ledger::new()))));
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/build/use_builtins.wasm"
    ));
    let exp_err = "RuntimeError:";

    match run_main(module, 10_000_000, &*interface) {
        Err(e) => {
            assert_eq!(
                "Sat Feb 01 2020 00:00:00",
                String::from_utf8(interface.raw_get_data("").expect("unexpected fail")).unwrap()
            );
            let err = e.to_string();
            assert!(err.starts_with(&format!("{exp_err} abort main, use_builtins.ts")));
        }
        _ => panic!("main was expected to fail"),
    }

    run_function(module, 10_000_000, "use_trace_1", "", &*interface)
        .expect("success expected for use_trace_1");
    assert_eq!(
        "hello world",
        String::from_utf8(interface.raw_get_data("").expect("unexpected fail")).unwrap()
    );

    run_function(module, 10_000_000, "use_trace_2", "", &*interface)
        .expect("success expected for use_trace_2");
    assert_eq!(
        "hello world, 0.1, 0.2, 0.3, 0.4, 0.5",
        String::from_utf8(interface.raw_get_data("").expect("unexpected fail")).unwrap()
    );

    match run_function(module, 10_000_000, "abort_1", "", &*interface) {
        Err(e) => {
            let err = e.to_string();
            assert!(err.starts_with(&format!("{exp_err} abort 1")));
        }
        _ => panic!("abort 1 was expected to fail"),
    }

    match run_function(module, 10_000_000, "abort_2", "", &*interface) {
        Err(e) => {
            let err = e.to_string();
            assert!(err.starts_with(&format!("{exp_err} abort 2, blop, line 2")));
        }
        _ => panic!("abort 2 was expected to fail"),
    }

    match run_function(module, 10_000_000, "abort_3", "", &*interface) {
        Err(e) => {
            let err = e.to_string();
            assert!(err.starts_with(&format!("{exp_err} abort 3, blop, line 2, col 3")));
        }
        _ => panic!("abort 3 was expected to fail"),
    }
}
