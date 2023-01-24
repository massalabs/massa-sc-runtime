/// THIS FILE SHOULD TEST THE ABI, NOT THE MOCKED INTERFACE
use crate::{
    run_function, run_main,
    types::{GasCosts, Interface},
    RuntimeModule,
};
use parking_lot::Mutex;
use rand::Rng;
use serial_test::serial;
use std::sync::Arc;

use crate::tests::{Ledger, TestInterface};

#[test]
#[serial]
fn test_caller() {
    let gas_costs = GasCosts::default();

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
    let runtime_module = RuntimeModule::new(&module, 200_000, gas_costs.clone()).unwrap();
    // test only if the module is valid
    run_main(&*interface, runtime_module, 20_000, gas_costs.clone())
        .expect("Failed to run_main get_string.wasm");
    let mut module = vec![1u8];
    module.extend_from_slice(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/build/caller.wasm"
    )));
    let runtime_module = RuntimeModule::new(&module, 200_000, gas_costs.clone()).unwrap();
    let a = run_main(
        &*interface,
        runtime_module.clone(),
        200_000,
        gas_costs.clone(),
    )
    .expect("Failed to run_main caller.wasm");
    //TODO: understand what's going on here
    //let prev_call_price = settings::metering_call();
    //settings::set_metering(0);
    let b = run_main(
        &*interface,
        runtime_module.clone(),
        200_000,
        gas_costs.clone(),
    )
    .expect("Failed to run_main caller.wasm");
    //assert_eq!(a + prev_call_price, b);
    assert_eq!(a.remaining_gas, b.remaining_gas);
    let v_out = interface.raw_get_data(b"").unwrap();
    let output = std::str::from_utf8(&v_out).unwrap();
    assert_eq!(output, "hello you");

    // Test now if we failed if metering is too high

    run_main(&*interface, runtime_module, 20_000, gas_costs.clone())
        .expect_err("Expected to be out of operation gas");
}

#[test]
#[serial]
fn test_caller_no_return() {
    let gas_costs = GasCosts::default();

    let interface: Box<dyn Interface> =
        Box::new(TestInterface(Arc::new(Mutex::new(Ledger::new()))));
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/build/get_string.wasm"
    ));
    interface.create_module(module.as_ref()).unwrap();
    let runtime_module = RuntimeModule::new(module, 200_000, gas_costs.clone()).unwrap();
    // test only if the module is valid
    run_main(&*interface, runtime_module, 20_000, gas_costs.clone())
        .expect("Failed to run get_string.wasm");
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/build/caller_no_return.wasm"
    ));
    let runtime_module = RuntimeModule::new(module, 200_000, gas_costs.clone()).unwrap();
    run_main(&*interface, runtime_module, 200_000, gas_costs.clone())
        .expect("Failed to run caller.wasm");
}

#[test]
#[serial]
fn test_local_hello_name_caller() {
    // This test should verify that even if we failed to load a module,
    // we should never panic and just stop the call stack
    let gas_costs = GasCosts::default();
    let interface: Box<dyn Interface> =
        Box::new(TestInterface(Arc::new(Mutex::new(Ledger::new()))));
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/build/get_string.wasm"
    ));
    interface
        .raw_set_bytecode_for("get_string", module.as_ref())
        .unwrap();
    let runtime_module = RuntimeModule::new(module, 200_000, gas_costs.clone()).unwrap();
    run_main(&*interface, runtime_module, 100_000, gas_costs.clone())
        .expect("Failed to run_main get_string.wasm");
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/build/local_hello_name_caller.wasm"
    ));
    let runtime_module = RuntimeModule::new(module, 200_000, gas_costs.clone()).unwrap();
    run_main(&*interface, runtime_module, 20_000, gas_costs.clone())
        .expect_err("Succeeded to run_main local_hello_name_caller.wasm");
}

#[test]
#[serial]
#[ignore]
fn test_module_creation() {
    let gas_costs = GasCosts::default();
    // This test should create a smartcontract module and call it
    let interface: Box<dyn Interface> =
        Box::new(TestInterface(Arc::new(Mutex::new(Ledger::new()))));
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/build/create_sc.wasm"
    ));
    let runtime_module = RuntimeModule::new(module, 200_000, gas_costs.clone()).unwrap();
    run_main(&*interface, runtime_module, 100_000, gas_costs.clone())
        .expect("Failed to run_main create_sc.wasm");
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/build/caller.wasm"
    ));
    let runtime_module = RuntimeModule::new(module, 200_000, gas_costs.clone()).unwrap();
    run_main(&*interface, runtime_module, 200_000, gas_costs.clone())
        .expect("Failed to run_main caller.wasm");
}

#[test]
#[serial]
#[ignore]
fn test_not_enough_gas_error() {
    let gas_costs = GasCosts::default();
    // This test should create a smartcontract module and call it
    let interface: Box<dyn Interface> =
        Box::new(TestInterface(Arc::new(Mutex::new(Ledger::new()))));
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/build/create_sc.wasm"
    ));
    let runtime_module = RuntimeModule::new(module, 200_000, gas_costs.clone()).unwrap();
    run_main(&*interface, runtime_module, 100_000, gas_costs.clone())
        .expect("Failed to run_main create_sc.wasm");
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/build/caller.wasm"
    ));
    let runtime_module = RuntimeModule::new(module, 200_000, gas_costs.clone()).unwrap();
    match run_main(&*interface, runtime_module, 50_000, gas_costs.clone()) {
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
    let gas_costs = GasCosts::default();
    let interface: Box<dyn Interface> =
        Box::new(TestInterface(Arc::new(Mutex::new(Ledger::new()))));
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/build/send_message.wasm"
    ));
    let runtime_module = RuntimeModule::new(module, 200_000, gas_costs.clone()).unwrap();
    run_main(&*interface, runtime_module, 100_000, gas_costs.clone())
        .expect("Failed to run_main send_message.wasm");
}

#[test]
#[serial]
fn test_run_function() {
    let gas_costs = GasCosts::default();
    let interface: Box<dyn Interface> =
        Box::new(TestInterface(Arc::new(Mutex::new(Ledger::new()))));
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/build/receive_message.wasm"
    ));
    let runtime_module = RuntimeModule::new(module, 200_000, gas_costs.clone()).unwrap();
    run_function(
        &*interface,
        runtime_module,
        "receive",
        b"data",
        200_000,
        gas_costs.clone(),
    )
    .expect("Failed to run_function receive_message.wasm");
}

#[test]
#[serial]
fn test_run_main_without_main() {
    let gas_costs = GasCosts::default();
    let interface: Box<dyn Interface> =
        Box::new(TestInterface(Arc::new(Mutex::new(Ledger::new()))));
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/build/no_main.wasm"
    ));
    let runtime_module = RuntimeModule::new(module, 200_000, gas_costs.clone()).unwrap();
    run_main(&*interface, runtime_module, 100_000, gas_costs)
        .expect_err("An error should spawn here");
}

#[test]
#[serial]
fn test_run_empty_main() {
    let mut gas_costs = GasCosts::default();
    let interface: Box<dyn Interface> =
        Box::new(TestInterface(Arc::new(Mutex::new(Ledger::new()))));
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/build/empty_main.wasm"
    ));
    // Even if our SC is empty; there is still an initial and minimum metering cost
    // (mainly because we have a memory allocator to init)
    gas_costs.launch_cost = 0;
    let runtime_module = RuntimeModule::new(module, 200_000, gas_costs.clone()).unwrap();
    let a = run_main(
        &*interface,
        runtime_module.clone(),
        10_000_000,
        gas_costs.clone(),
    )
    .expect("Failed to run empty_main.wasm");
    // Here we avoid hard-coding a value (that can change in future wasmer release)$
    assert!(a.remaining_gas > 0);

    let mut rng = rand::thread_rng();
    let cost = rng.gen_range(1..1_000_000);
    gas_costs.launch_cost = cost;
    let b = run_main(&*interface, runtime_module, 10_000_000, gas_costs)
        .expect("Failed to run empty_main.wasm");
    // Between 2 calls, the metering cost should be the difference
    assert_eq!(a.remaining_gas - b.remaining_gas, cost);
}

#[test]
#[serial]
fn test_op_fn() {
    let gas_costs = GasCosts::default();
    let interface: Box<dyn Interface> =
        Box::new(TestInterface(Arc::new(Mutex::new(Ledger::new()))));
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/build/op_fn.wasm"
    ));
    let runtime_module = RuntimeModule::new(module, 200_000, gas_costs.clone()).unwrap();
    run_main(&*interface, runtime_module, 10_000_000, gas_costs.clone())
        .expect("Failed to run op_fn.wasm");
}

/// Test seed, now and abort
#[test]
#[serial]
fn test_builtins() {
    let gas_costs = GasCosts::default();
    let interface: Box<dyn Interface> =
        Box::new(TestInterface(Arc::new(Mutex::new(Ledger::new()))));
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/build/use_builtins.wasm"
    ));
    let runtime_module = RuntimeModule::new(module, 200_000, gas_costs.clone()).unwrap();
    match run_main(&*interface, runtime_module, 10_000_000, gas_costs.clone()) {
        Err(e) => {
            println!("Error: {}", e);
            assert!(e.to_string().starts_with(
                "RuntimeError: Runtime error: error: abord with date and rnd at use_builtins.ts"
            ));
        }
        _ => panic!("Failed to run use_builtins.wasm"),
    }
}

#[test]
#[serial]
fn test_wat() {
    let gas_costs = GasCosts::default();
    let interface: Box<dyn Interface> =
        Box::new(TestInterface(Arc::new(Mutex::new(Ledger::new()))));
    let bytecode = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/wasm/build/dummy.wat"));

    let gas_limit = 100_000;
    let runtime_module = RuntimeModule::new(bytecode, gas_limit, gas_costs.clone()).unwrap();
    let response = run_main(&*interface, runtime_module, gas_limit, gas_costs.clone()).unwrap();

    // Note: for now, exec main always return an empty vec
    let excepted: Vec<u8> = Vec::new();
    assert_eq!(response.ret, excepted);
}

/// Test wasm using features disabled in engine (simd & threads)
#[test]
#[serial]
fn test_features_disabled() {
    let gas_costs = GasCosts::default();
    let interface: Box<dyn Interface> =
        Box::new(TestInterface(Arc::new(Mutex::new(Ledger::new()))));

    let module = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/wasm/build/simd.wasm"));
    match run_main(module, 10_000_000, &*interface, gas_costs.clone()) {
        Err(e) => {
            // println!("Error: {}", e);
            assert!(e
                .to_string()
                .starts_with("Validation error: SIMD support is not enabled"));
        }
        _ => panic!("Failed to run use_builtins.wasm"),
    }

    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/build/threads.wasm"
    ));
    match run_main(module, 10_000_000, &*interface, gas_costs) {
        Err(e) => {
            // println!("Error: {}", e);
            assert!(e
                .to_string()
                .starts_with("Validation error: threads support is not enabled"));
        }
        _ => panic!("Failed to run use_builtins.wasm"),
    }
}
