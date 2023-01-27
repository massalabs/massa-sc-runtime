use crate::tests::{Ledger, TestInterface};
use crate::{
    run_function, run_main,
    types::{GasCosts, Interface},
    RuntimeModule,
};
use parking_lot::Mutex;
use rand::Rng;
use serial_test::serial;
use std::sync::Arc;

#[test]
#[serial]
/// Test basic main-only SC execution
fn test_run_main() {
    let gas_costs = GasCosts::default();
    let interface: Box<dyn Interface> =
        Box::new(TestInterface(Arc::new(Mutex::new(Ledger::new()))));
    let module = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/wasm/basic_main.wasm"));

    let runtime_module = RuntimeModule::new(module, 200_000, gas_costs.clone()).unwrap();
    run_main(&*interface, runtime_module, 100_000, gas_costs).unwrap();
}

#[test]
#[serial]
/// Test basic function-only SC execution
fn test_run_function() {
    let gas_costs = GasCosts::default();
    let interface: Box<dyn Interface> =
        Box::new(TestInterface(Arc::new(Mutex::new(Ledger::new()))));
    let module = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/wasm/basic_func.wasm"));

    let runtime_module = RuntimeModule::new(module, 200_000, gas_costs.clone()).unwrap();
    run_function(&*interface, runtime_module, "ping", b"", 100_000, gas_costs).unwrap();
}

#[test]
#[serial]
/// Test both cases of the not enough gas error
fn test_not_enough_gas_error() {
    let gas_costs = GasCosts::default();
    let interface: Box<dyn Interface> =
        Box::new(TestInterface(Arc::new(Mutex::new(Ledger::new()))));
    let module = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/wasm/basic_main.wasm"));

    // Test giving not enough gas to create the instance
    let runtime_module = RuntimeModule::new(module, 100, gas_costs.clone()).unwrap();
    let error = run_main(&*interface, runtime_module, 100_000, gas_costs.clone())
        .unwrap_err()
        .to_string();
    assert!(
        error == "Not enough gas, limit reached at initialization"
            || error.contains("RuntimeError: unreachable")
    );

    // Test giving enough gas to create the instance but not enough for the VM
    let runtime_module = RuntimeModule::new(module, 100_000, gas_costs.clone()).unwrap();
    let error = run_main(&*interface, runtime_module, 100, gas_costs).unwrap_err();
    assert_eq!(
        error.to_string(),
        "Not enough gas to launch the virtual machine"
    );
}

#[test]
#[serial]
/// Test that a no-main SC executed through `run_main` fails as expected
fn test_run_main_without_main() {
    let gas_costs = GasCosts::default();
    let interface: Box<dyn Interface> =
        Box::new(TestInterface(Arc::new(Mutex::new(Ledger::new()))));
    let module = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/wasm/no_main.wasm"));
    let runtime_module = RuntimeModule::new(module, 200_000, gas_costs.clone()).unwrap();
    run_main(&*interface, runtime_module, 100_000, gas_costs)
        .expect_err("An error should spawn here");
}

#[test]
#[serial]
/// Even if our SC is empty there is still an initial and minimum metering cost,
/// because we have a memory allocator to init.
///
/// This test ensure that this initial cost is correctly debited.
fn test_run_empty_main() {
    let mut gas_costs = GasCosts::default();
    let interface: Box<dyn Interface> =
        Box::new(TestInterface(Arc::new(Mutex::new(Ledger::new()))));
    let module = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/wasm/empty_main.wasm"));
    gas_costs.launch_cost = 0;
    let runtime_module = RuntimeModule::new(module, 200_000, gas_costs.clone()).unwrap();
    let a = run_main(
        &*interface,
        runtime_module.clone(),
        10_000_000,
        gas_costs.clone(),
    )
    .expect("Failed to run empty_main.wasm");
    // Here we avoid hard-coding a value (that can change in future wasmer release)
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
/// Test the operation datastore
///
/// * getOpKeys
/// * hasOpKey
/// * getOpData
fn test_op_fn() {
    let gas_costs = GasCosts::default();
    let interface: Box<dyn Interface> =
        Box::new(TestInterface(Arc::new(Mutex::new(Ledger::new()))));
    let module = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/wasm/op_fn.wasm"));
    let runtime_module = RuntimeModule::new(module, 200_000, gas_costs.clone()).unwrap();
    run_main(&*interface, runtime_module, 10_000_000, gas_costs.clone())
        .expect("Failed to run op_fn.wasm");
}

/// Test `seed`, `Date.now`, `console.log` and `abort`
///
/// These are AS functions that we choose to handle in the VM
#[test]
#[serial]
fn test_builtins() {
    let gas_costs = GasCosts::default();
    let interface: Box<dyn Interface> =
        Box::new(TestInterface(Arc::new(Mutex::new(Ledger::new()))));
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/use_builtins.wasm"
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
/// Ensure that the execution of WAT files (text equivalent of WASM) is supported
///
/// WAT files are mostly used in testing
fn test_wat() {
    let gas_costs = GasCosts::default();
    let interface: Box<dyn Interface> =
        Box::new(TestInterface(Arc::new(Mutex::new(Ledger::new()))));
    let bytecode = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/wasm/dummy.wat"));

    let gas_limit = 100_000;
    let runtime_module = RuntimeModule::new(bytecode, gas_limit, gas_costs.clone()).unwrap();
    let response = run_main(&*interface, runtime_module, gas_limit, gas_costs.clone()).unwrap();

    // Note: for now, exec main always return an empty vec
    let excepted: Vec<u8> = Vec::new();
    assert_eq!(response.ret, excepted);
}

/// Test a WASM execution using features disabled in engine (simd & threads)
#[test]
#[serial]
fn test_features_disabled() {
    let gas_costs = GasCosts::default();

    let module = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/wasm/simd.wasm"));
    match RuntimeModule::new(module, 200_000, gas_costs.clone()) {
        Err(e) => {
            // println!("Error: {}", e);
            assert!(e
                .to_string()
                .starts_with("Validation error: SIMD support is not enabled"));
        }
        _ => panic!("Failed to run use_builtins.wasm"),
    }

    let module = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/wasm/threads.wasm"));
    match RuntimeModule::new(module, 200_000, gas_costs.clone()) {
        Err(e) => {
            // println!("Error: {}", e);
            assert!(e
                .to_string()
                .starts_with("Validation error: threads support is not enabled"));
        }
        _ => panic!("Failed to run use_builtins.wasm"),
    }
}
