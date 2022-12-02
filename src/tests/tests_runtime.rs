/// THIS FILE SHOULD TEST THE ABI, NOT THE MOCKED INTERFACE
use crate::{run_function, run_main, settings, types::Interface};
use parking_lot::Mutex;
use rand::Rng;
use serial_test::serial;
use std::sync::Arc;

use crate::tests::{Ledger, TestInterface};

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
    let v_out = interface.raw_get_data(b"").unwrap();
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
#[ignore]
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
#[ignore]
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
    run_function(module, 100_000, "receive", b"data", &*interface)
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
    // Here we avoid hard-coding a value (that can change in future wasmer release)
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

/// Test seed, now and abort
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
    match run_main(module, 10_000_000, &*interface) {
        Err(e) => {
            println!("Error: {}", e);
            assert!(e
                .to_string()
                .starts_with("RuntimeError: error: abord with date and rnd at use_builtins.ts"));
        }
        _ => panic!("Failed to run use_builtins.wasm"),
    }
}
