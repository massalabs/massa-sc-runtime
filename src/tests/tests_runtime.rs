use crate::as_execution::{init_store, ASContext, ASModule};
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
use wasmer::WasmPtr;

#[test]
#[serial]
/// Test that overriding the metering globals is not possible
fn test_metering_safety() {
    let interface: TestInterface = TestInterface(Arc::new(Mutex::new(Ledger::new())));
    let bytecode = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/metering_override.wat"
    ));

    let gas_costs = GasCosts::default();
    let runtime_module = RuntimeModule::new(bytecode, 100_000, gas_costs.clone(), false).unwrap();
    let resp = run_main(&interface, runtime_module, 100_000, gas_costs.clone()).unwrap();
    assert_ne!(resp.remaining_gas, 42);
}

#[test]
#[serial]
/// Test that calling ABIs from the start function is not possible
fn test_instantiation_safety() {
    let interface: TestInterface = TestInterface(Arc::new(Mutex::new(Ledger::new())));
    let bytecode = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/start_func_abi_call.wat"
    ));

    let gas_costs = GasCosts::default();
    let runtime_module = RuntimeModule::new(bytecode, 100_000, gas_costs.clone(), false).unwrap();
    let error = run_main(&interface, runtime_module, 100_000, gas_costs.clone()).unwrap_err();
    let expected_error = "ABI calls are not available during instantiation";
    assert!(error.to_string().contains(expected_error));
}

#[test]
#[serial]
/// Test basic main-only SC execution
fn test_run_main() {
    let gas_costs = GasCosts::default();
    let interface: Box<dyn Interface> =
        Box::new(TestInterface(Arc::new(Mutex::new(Ledger::new()))));
    let module = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/wasm/basic_main.wasm"));

    let runtime_module = RuntimeModule::new(module, 200_000, gas_costs.clone(), false).unwrap();
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

    let runtime_module = RuntimeModule::new(module, 200_000, gas_costs.clone(), false).unwrap();
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
    let runtime_module = RuntimeModule::new(module, 100, gas_costs.clone(), false).unwrap();
    let error = run_main(&*interface, runtime_module, 100_000, gas_costs.clone())
        .unwrap_err()
        .to_string();
    assert!(
        error == "Not enough gas, limit reached at initialization"
            || error.contains("RuntimeError: unreachable")
    );

    // Test giving enough gas to create the instance but not enough for the VM
    let runtime_module = RuntimeModule::new(module, 100_000, gas_costs.clone(), false).unwrap();
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
    let runtime_module = RuntimeModule::new(module, 200_000, gas_costs.clone(), false).unwrap();
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
    let runtime_module = RuntimeModule::new(module, 200_000, gas_costs.clone(), false).unwrap();
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
    let runtime_module = RuntimeModule::new(module, 200_000, gas_costs.clone(), false).unwrap();
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
    let runtime_module = RuntimeModule::new(module, 200_000, gas_costs.clone(), false).unwrap();
    let before = chrono::offset::Utc::now().timestamp_millis();
    match run_main(&*interface, runtime_module, 10_000_000, gas_costs.clone()) {
        Err(e) => {
            let msg = e.to_string();
            // make sure the error was caused by a manual abort
            assert!(msg.contains("Manual abort"));
            // check the given timestamp validity
            let after = chrono::offset::Utc::now().timestamp_millis();
            let ident = "UTC timestamp (ms) = ";
            let start = msg.find(ident).unwrap_or(0).saturating_add(ident.len());
            let end = msg.find(" at use_builtins.ts").unwrap_or(0);
            let sc_timestamp: i64 = msg[start..end].parse().unwrap();
            assert!(before <= sc_timestamp && sc_timestamp <= after);
        }
        _ => panic!("Failed to run use_builtins.wasm"),
    }
}

#[test]
#[serial]
/// Test `assert` & `process.exit
///
/// These are AS functions that we choose to handle in the VM
fn test_builtin_assert_and_exit() {
    let gas_costs = GasCosts::default();
    let interface: Box<dyn Interface> =
        Box::new(TestInterface(Arc::new(Mutex::new(Ledger::new()))));
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/use_builtin_assert.wasm"
    ));

    let runtime_module = RuntimeModule::new(module, 200_000, gas_costs.clone(), false).unwrap();
    match run_function(
        &*interface,
        runtime_module,
        "assert_with_msg",
        b"",
        100_000,
        gas_costs.clone(),
    ) {
        Err(e) => {
            assert!(e.to_string().contains("Result is not true!"))
        }
        _ => panic!("test should return an error!"),
    }

    let runtime_module = RuntimeModule::new(module, 200_000, gas_costs.clone(), false).unwrap();
    if let Ok(_) = run_function(
        &*interface,
        runtime_module,
        "assert_no_msg",
        b"",
        100_000,
        gas_costs.clone(),
    ) {
        panic!("test should return an error!");
    }

    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/use_builtin_exit.wasm"
    ));

    let runtime_module = RuntimeModule::new(module, 200_000, gas_costs.clone(), false).unwrap();
    match run_function(
        &*interface,
        runtime_module,
        "exit_no_code",
        b"",
        100_000,
        gas_costs.clone(),
    ) {
        Err(e) => {
            assert!(e.to_string().contains("exit with code: 0"))
        }
        _ => panic!("test should return an error!"),
    }

    let runtime_module = RuntimeModule::new(module, 200_000, gas_costs.clone(), false).unwrap();
    match run_function(
        &*interface,
        runtime_module,
        "exit_with_code",
        b"",
        100_000,
        gas_costs,
    ) {
        Err(e) => {
            assert!(e.to_string().contains("exit with code: 2"))
        }
        _ => panic!("test should return an error!"),
    }
}

#[test]
#[serial]
/// Test WASM files compiled with unsupported builtin functions
fn test_unsupported_builtins() {
    let gas_costs = GasCosts::default();
    let interface: Box<dyn Interface> =
        Box::new(TestInterface(Arc::new(Mutex::new(Ledger::new()))));
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/unsupported_builtin_hrtime.wasm"
    ));
    let runtime_module = RuntimeModule::new(module, 200_000, gas_costs.clone(), false).unwrap();

    match run_main(&*interface, runtime_module, 10_000_000, gas_costs.clone()) {
        Err(e) => {
            assert!(e
                .to_string()
                .contains("Error while importing \"env\".\"performance.now\""))
        }
        _ => panic!("test should return an error!"),
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
    let runtime_module = RuntimeModule::new(bytecode, gas_limit, gas_costs.clone(), false).unwrap();
    let response = run_main(&*interface, runtime_module, gas_limit, gas_costs.clone()).unwrap();

    // Note: for now, exec main always return an empty vec
    let excepted: Vec<u8> = Vec::new();
    assert_eq!(response.ret, excepted);
}

#[test]
#[serial]
/// Test a WASM execution using features disabled in engine (simd & threads)
fn test_features_disabled() {
    let gas_costs = GasCosts::default();

    let module = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/wasm/simd.wasm"));
    match RuntimeModule::new(module, 200_000, gas_costs.clone(), false) {
        Err(e) => {
            // println!("Error: {}", e);
            assert!(e
                .to_string()
                .starts_with("Validation error: SIMD support is not enabled"));
        }
        _ => panic!("Failed to run use_builtins.wasm"),
    }

    let module = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/wasm/threads.wasm"));
    match RuntimeModule::new(module, 200_000, gas_costs.clone(), false) {
        Err(e) => {
            // println!("Error: {}", e);
            assert!(e
                .to_string()
                .starts_with("Validation error: threads support is not enabled"));
        }
        _ => panic!("Failed to run use_builtins.wasm"),
    }
}

#[test]
#[serial]
/// Non regression test on the AS class id values
fn test_class_id() {
    // setup basic AS runtime context
    let interface: Box<dyn Interface> =
        Box::new(TestInterface(Arc::new(Mutex::new(Ledger::new()))));
    let bytecode = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/return_basic.wasm"
    ));
    let module = ASModule::new(bytecode, 100_000, GasCosts::default(), false).unwrap();
    let mut store = init_store(&module.engine).unwrap();
    let mut context = ASContext::new(&*interface, module.binary_module, GasCosts::default());
    let (instance, _) = context.create_vm_instance_and_init_env(&mut store).unwrap();

    // setup test specific context
    let (_, fenv) = context.resolver(&mut store);
    let fenv_mut = fenv.into_mut(&mut store);
    let memory = context.env.get_ffi_env().memory.as_ref().unwrap();
    let memory_view = memory.view(&fenv_mut);

    // get string and array offsets
    let return_string = instance.exports.get_function("return_string").unwrap();
    let return_array = instance.exports.get_function("return_array").unwrap();
    let string_ptr = return_string
        .call(&mut store, &[])
        .unwrap()
        .get(0)
        .unwrap()
        .i32()
        .unwrap();
    let array_ptr = return_array
        .call(&mut store, &[])
        .unwrap()
        .get(0)
        .unwrap()
        .i32()
        .unwrap();

    // use `u32` size to retrieve the class id
    // see https://www.assemblyscript.org/runtime.html#memory-layout
    let u32_size = std::mem::size_of::<u32>() as u32;

    // read and assert string class id
    let string_w_ptr: WasmPtr<u8> = WasmPtr::new(string_ptr as u32)
        .sub_offset(u32_size * 2)
        .unwrap();
    let slice_len_buf = string_w_ptr
        .slice(&memory_view, u32_size)
        .unwrap()
        .read_to_vec()
        .unwrap();
    let string_class_id = u32::from_ne_bytes(
        slice_len_buf
            .try_into()
            .map_err(|_| wasmer::RuntimeError::new("Unable to convert vec to [u8; 4]"))
            .unwrap(),
    );
    assert_eq!(string_class_id, 2);

    // read and assert array class id
    let array_w_ptr: WasmPtr<u8> = WasmPtr::new(array_ptr as u32)
        .sub_offset(u32_size * 2)
        .unwrap();
    let slice_len_buf = array_w_ptr
        .slice(&memory_view, u32_size)
        .unwrap()
        .read_to_vec()
        .unwrap();
    let array_class_id = u32::from_ne_bytes(
        slice_len_buf
            .try_into()
            .map_err(|_| wasmer::RuntimeError::new("Unable to convert vec to [u8; 4]"))
            .unwrap(),
    );
    assert_eq!(array_class_id, 4);
}
