use crate::as_execution::{ASContext, ASModule};
use crate::tests::TestInterface;
use crate::{
    run_function, run_main,
    types::{GasCosts, Interface},
    RuntimeModule,
};
use crate::{Compiler, CondomLimits};
use rand::Rng;
use serial_test::serial;
use wasmer::Store;
use wasmer::WasmPtr;

#[cfg(feature = "execution-trace")]
use crate::{AbiTrace, AbiTraceType, AbiTraceValue};

#[test]
#[serial]
#[ignore]
/// Test exhaustive smart contract
fn test_exhaustive_smart_contract() {
    let interface = TestInterface;
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/test_exhaustive_smart_contract.wasm_add"
    ));
    let gas_costs = GasCosts::default();
    let condom_limits = CondomLimits::default();

    let runtime_module = RuntimeModule::new(
        module,
        gas_costs.clone(),
        Compiler::SP,
        condom_limits.clone(),
    )
    .unwrap();
    run_main(
        &interface,
        runtime_module,
        100_000_000,
        gas_costs,
        condom_limits,
    )
    .unwrap();
}

#[test]
#[serial]
/// Test native time arithmetic ABI calls
fn test_native_time_arithmetic_abis() {
    let interface = TestInterface;
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/test_native_time_arithmetic.wasm_add"
    ));
    let gas_costs = GasCosts::default();
    let condom_limits = CondomLimits::default();

    let runtime_module = RuntimeModule::new(
        module,
        gas_costs.clone(),
        Compiler::SP,
        condom_limits.clone(),
    )
    .unwrap();
    run_main(
        &interface,
        runtime_module,
        100_000_000,
        gas_costs,
        condom_limits,
    )
    .unwrap();
}

#[test]
#[serial]
/// Test structs check and version ABI calls
fn test_structs_check_and_version_abis() {
    let interface = TestInterface;
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/test_structs_check_and_version.wasm_add"
    ));
    let gas_costs = GasCosts::default();
    let condom_limits = CondomLimits::default();

    let runtime_module = RuntimeModule::new(
        module,
        gas_costs.clone(),
        Compiler::SP,
        condom_limits.clone(),
    )
    .unwrap();
    run_main(
        &interface,
        runtime_module,
        100_000_000,
        gas_costs,
        condom_limits,
    )
    .unwrap();
}

#[test]
#[serial]
/// Test datastore ABI calls
fn test_datastore_abis() {
    let interface = TestInterface;
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/test_datastore.wasm_add"
    ));
    let gas_costs = GasCosts::default();
    let condom_limits = CondomLimits::default();

    let runtime_module = RuntimeModule::new(
        module,
        gas_costs.clone(),
        Compiler::SP,
        condom_limits.clone(),
    )
    .unwrap();
    run_main(
        &interface,
        runtime_module,
        100_000_000,
        gas_costs,
        condom_limits,
    )
    .unwrap();
}

#[test]
#[serial]
/// Test ledger and op keys ABI calls
fn test_ledger_op_keys_abis() {
    let interface = TestInterface;
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/test_ledger_op_keys.wasm_add"
    ));
    let gas_costs = GasCosts::default();
    let condom_limits = CondomLimits::default();

    let runtime_module = RuntimeModule::new(
        module,
        gas_costs.clone(),
        Compiler::SP,
        condom_limits.clone(),
    )
    .unwrap();
    run_main(
        &interface,
        runtime_module,
        100_000_000,
        gas_costs,
        condom_limits,
    )
    .unwrap();
}

#[test]
#[serial]
/// Test that overriding the metering globals is not possible
fn test_metering_safety() {
    let interface = TestInterface;
    let bytecode = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/metering_override.wasm"
    ));

    let gas_costs = GasCosts::default();
    let condom_limits = CondomLimits::default();
    let runtime_module = RuntimeModule::new(
        bytecode,
        gas_costs.clone(),
        Compiler::SP,
        condom_limits.clone(),
    )
    .unwrap();
    let resp = run_main(
        &interface,
        runtime_module,
        100_000,
        gas_costs.clone(),
        condom_limits.clone(),
    )
    .unwrap();
    assert_ne!(resp.remaining_gas, 42);
}

#[test]
#[serial]
/// Test that calling ABIs from the start function is not possible
fn test_instantiation_safety() {
    let interface = TestInterface;
    let bytecode = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/start_func_abi_call.wasm"
    ));

    let gas_costs = GasCosts::default();
    let condom_limits = CondomLimits::default();
    let runtime_module = RuntimeModule::new(
        bytecode,
        gas_costs.clone(),
        Compiler::SP,
        condom_limits.clone(),
    )
    .unwrap();
    let error = run_main(
        &interface,
        runtime_module,
        100_000,
        gas_costs.clone(),
        condom_limits.clone(),
    )
    .unwrap_err();
    let expected_error = "ABI calls are not available during instantiation";
    assert!(error.to_string().contains(expected_error));
}

#[test]
#[serial]
/// Test basic main-only SC execution
fn test_run_main() {
    let gas_costs = GasCosts::default();
    let condom_limits = CondomLimits::default();
    let interface: Box<dyn Interface> = Box::new(TestInterface);
    let module = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/wasm/basic_main.wasm"));

    let runtime_module = RuntimeModule::new(
        module,
        gas_costs.clone(),
        Compiler::SP,
        condom_limits.clone(),
    )
    .unwrap();
    run_main(
        &*interface,
        runtime_module,
        100_000,
        gas_costs,
        condom_limits,
    )
    .unwrap();
}

#[cfg(feature = "execution-trace")]
#[test]
#[serial]
/// Test basic main-only SC execution
fn test_run_main_get_execution_traces() {
    let gas_costs = GasCosts::default();
    let condom_limits = CondomLimits::default();
    let interface: Box<dyn Interface> = Box::new(TestInterface);
    let module = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/wasm/basic_main.wasm"));

    let runtime_module = RuntimeModule::new(
        module,
        gas_costs.clone(),
        Compiler::SP,
        condom_limits.clone(),
    )
    .unwrap();
    let resp = run_main(
        &*interface,
        runtime_module,
        100_000,
        gas_costs,
        condom_limits,
    )
    .unwrap();

    assert_eq!(resp.trace.is_empty(), false);
    assert_eq!(
        resp.trace,
        vec![AbiTrace {
            name: "assembly_script_generate_event".to_string(),
            params: vec![("event", "hello world!".to_string()).into()],
            return_value: AbiTraceType::None,
            sub_calls: None
        }]
    )
}

#[test]
#[serial]
fn test_run_register_wasmv1() {
    // let gas_costs = GasCosts::default();
    // let condom_limits = CondomLimits::default();
    // let interface: Box<dyn Interface> =
    //     Box::new(TestInterface);
    // let module = include_bytes!(concat!(
    //     env!("CARGO_MANIFEST_DIR"),
    //     "/../abi_as/build/release.wasm_add"
    // ));

    // let runtime_module =
    //     RuntimeModule::new(module, gas_costs.clone(),
    // Compiler::SP, condom_limits.clone()))?;.unwrap();

    // match runtime_module.clone() {
    //     RuntimeModule::ASModule(_) => {
    //         println!("Module type ASModule");
    //     }
    //     RuntimeModule::WasmV1Module(_) => {
    //         println!("Module type WasmV1Module");
    //     }
    // }

    // let res = run_function(
    //     &*interface,
    //     runtime_module,
    //     "initialize",
    //     b"",
    //     100_000,
    //     gas_costs,
    // )
    // .unwrap();
    // println!("{:?}", res);
}

// #[test]
// #[serial]
/// Test basic main-only SC execution
// fn test_run_main_wasmv1() {
//     let gas_costs = GasCosts::default();
//     let interface: Box<dyn Interface> = Box::new(TestInterface);
//     let module = include_bytes!(concat!(
//         env!("CARGO_MANIFEST_DIR"),
//         "/wasm/dmain.wasm_add"
//     ));

//     let runtime_module =
//         RuntimeModule::new(module, gas_costs.clone(), Compiler::SP, condom_limits.clone())?;
//             .unwrap();

//     match runtime_module.clone() {
//         RuntimeModule::ASModule(_) => {
//             println!("Module type ASModule");
//         }
//         RuntimeModule::WasmV1Module(_) => {
//             println!("Module type WasmV1Module");
//         }
//     }
//     run_main(&*interface, runtime_module, 100_000_000, gas_costs, condom_limits).unwrap();
// }

#[test]
#[serial]
/// Test test_get_current_period_and_thread
fn test_get_current_period_and_thread_wasmv1_as() {
    let gas_costs = GasCosts::default();
    let condom_limits = CondomLimits::default();
    let interface: Box<dyn Interface> = Box::new(TestInterface);
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/test_period_thread.wasm_add"
    ));

    let runtime_module = RuntimeModule::new(
        module,
        gas_costs.clone(),
        Compiler::SP,
        condom_limits.clone(),
    )
    .unwrap();

    match runtime_module.clone() {
        RuntimeModule::ASModule(_) => {
            panic!("Error: Module type ASModule, expected WasmV1Module!");
        }
        RuntimeModule::WasmV1Module(_) => {
            println!("Module type WasmV1Module");
        }
    }
    run_main(
        &*interface,
        runtime_module,
        100_000_000,
        gas_costs,
        condom_limits,
    )
    .unwrap();
}

#[test]
#[serial]
/// Test test_native_hash
fn test_native_hash_wasmv1_as() {
    let gas_costs = GasCosts::default();
    let condom_limits = CondomLimits::default();
    let interface: Box<dyn Interface> = Box::new(TestInterface);
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/test_hash.wasm_add"
    ));

    let runtime_module = RuntimeModule::new(
        module,
        gas_costs.clone(),
        Compiler::SP,
        condom_limits.clone(),
    )
    .unwrap();

    match runtime_module.clone() {
        RuntimeModule::ASModule(_) => {
            panic!("Error: Module type ASModule, expected WasmV1Module!");
        }
        RuntimeModule::WasmV1Module(_) => {
            println!("Module type WasmV1Module");
        }
    }
    run_main(
        &*interface,
        runtime_module,
        100_000_000,
        gas_costs,
        condom_limits,
    )
    .unwrap();
}

#[test]
#[serial]
/// This test call the main function of a SC that calls generate_event abi
fn test_generate_event_wasmv1_as() {
    let gas_costs = GasCosts::default();
    let condom_limits = CondomLimits::default();
    let interface: Box<dyn Interface> = Box::new(TestInterface);
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/test_generate_event.wasm_add"
    ));

    let runtime_module = RuntimeModule::new(
        module,
        gas_costs.clone(),
        Compiler::SP,
        condom_limits.clone(),
    )
    .unwrap();

    match runtime_module.clone() {
        RuntimeModule::ASModule(_) => {
            panic!("Must be WasmV1Module");
        }
        RuntimeModule::WasmV1Module(_) => {
            println!("Module type WasmV1Module");
        }
    }
    run_main(
        &*interface,
        runtime_module,
        100_000_000,
        gas_costs,
        condom_limits,
    )
    .unwrap();
}

#[test]
#[serial]
/// This test arithmetic operations on native amount
fn test_native_amount_arithmetic_wasmv1_as() {
    let gas_costs = GasCosts::default();
    let condom_limits = CondomLimits::default();
    let interface: Box<dyn Interface> = Box::new(TestInterface);
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/test_native_amount_arithmetic.wasm_add"
    ));

    let runtime_module = RuntimeModule::new(
        module,
        gas_costs.clone(),
        Compiler::SP,
        condom_limits.clone(),
    )
    .unwrap();

    match runtime_module.clone() {
        RuntimeModule::ASModule(_) => {
            panic!("Must be WasmV1Module");
        }
        RuntimeModule::WasmV1Module(_) => {
            println!("Module type WasmV1Module");
        }
    }
    run_main(
        &*interface,
        runtime_module,
        100_000_000,
        gas_costs,
        condom_limits,
    )
    .unwrap();
}

#[test]
#[serial]
/// This test call the main function of a SC that will abort
fn test_abort_wasmv1_as() {
    let gas_costs = GasCosts::default();
    let condom_limits = CondomLimits::default();
    let interface: Box<dyn Interface> = Box::new(TestInterface);
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/test_abort.wasm_add"
    ));

    let runtime_module = RuntimeModule::new(
        module,
        gas_costs.clone(),
        Compiler::SP,
        condom_limits.clone(),
    )
    .unwrap();

    match runtime_module.clone() {
        RuntimeModule::ASModule(_) => {
            panic!("Must be WasmV1Module");
        }
        RuntimeModule::WasmV1Module(_) => {
            println!("Module type WasmV1Module");
        }
    }

    let res = run_main(
        &*interface,
        runtime_module,
        100_000,
        gas_costs,
        condom_limits,
    );

    match res {
        Err(e) if e.to_string().contains("abort test message") => {
            println!("Ok abort: {:?}", e);
            return;
        }
        Err(e) => {
            println!("Test failed: {:?}", e);
            panic!("Expected abort");
        }
        Ok(_) => {
            panic!("Err expected");
        }
    }
}

#[test]
#[serial]
/// This test call the main function of a SC that will abort
fn test_assert_in_release_wasmv1_as() {
    let gas_costs = GasCosts::default();
    let condom_limits = CondomLimits::default();
    let interface: Box<dyn Interface> = Box::new(TestInterface);
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/test_assert_in_release.wasm_add"
    ));

    let runtime_module = RuntimeModule::new(
        module,
        gas_costs.clone(),
        Compiler::SP,
        condom_limits.clone(),
    )
    .unwrap();

    match runtime_module.clone() {
        RuntimeModule::ASModule(_) => {
            panic!("Must be WasmV1Module");
        }
        RuntimeModule::WasmV1Module(_) => {
            println!("Module type WasmV1Module");
        }
    }

    let res = run_main(
        &*interface,
        runtime_module,
        100_000_000,
        gas_costs,
        condom_limits,
    );

    match res {
        Err(e) if e.to_string().contains("expected assert") => {
            println!("Ok program must assert: {:?}", e);
            return;
        }
        Err(e) => {
            println!("Test failed: {:?}", e);
            panic!("Expected abort");
        }
        Ok(_) => {
            panic!("Err expected");
        }
    }
}

#[test]
#[serial]
/// This test call the main function of a SC that calls transfer_coins abi
fn test_transfer_coins_wasmv1_as() {
    let gas_costs = GasCosts::default();
    let condom_limits = CondomLimits::default();
    let interface: Box<dyn Interface> = Box::new(TestInterface);
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/test_transfer_coins.wasm_add"
    ));

    let runtime_module = RuntimeModule::new(
        module,
        gas_costs.clone(),
        Compiler::SP,
        condom_limits.clone(),
    )
    .unwrap();

    match runtime_module.clone() {
        RuntimeModule::ASModule(_) => {
            panic!("Must be WasmV1Module");
        }
        RuntimeModule::WasmV1Module(_) => {
            println!("Module type WasmV1Module");
        }
    }

    let _resp = run_main(
        &*interface,
        runtime_module,
        100_000,
        gas_costs,
        condom_limits,
    )
    .unwrap();

    #[cfg(feature = "execution-trace")]
    {
        assert_eq!(_resp.trace.is_empty(), false);
        let trace_1 = _resp.trace.get(0).unwrap();
        assert_eq!(trace_1.name, "abi_transfer_coins");
        assert_eq!(
            trace_1.params,
            vec![
                ("target_address", "abcd".to_string()).into(),
                ("amount", 100i64).into(),
                ("sender_address", "efgh".to_string()).into(),
            ]
        );
        assert_eq!(trace_1.return_value, AbiTraceType::None);
        assert_eq!(trace_1.sub_calls, None);
    }
}

#[test]
#[serial]
/// This test call the main function of a SC that calls bs58 encode/decode abi
fn test_bs58_to_from_wasmv1_as() {
    let gas_costs = GasCosts::default();
    let condom_limits = CondomLimits::default();
    let interface: Box<dyn Interface> = Box::new(TestInterface);
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/test_bs58_to_from.wasm_add"
    ));

    let runtime_module = RuntimeModule::new(
        module,
        gas_costs.clone(),
        Compiler::SP,
        condom_limits.clone(),
    )
    .unwrap();

    match runtime_module.clone() {
        RuntimeModule::ASModule(_) => {
            panic!("Must be WasmV1Module");
        }
        RuntimeModule::WasmV1Module(_) => {
            println!("Module type WasmV1Module");
        }
    }
    run_main(
        &*interface,
        runtime_module,
        100_000_000,
        gas_costs,
        condom_limits,
    )
    .unwrap();
}

#[test]
#[serial]
/// This test call the main function of a SC that calls comparisons abis
fn test_compare_wasmv1_as() {
    let gas_costs = GasCosts::default();
    let condom_limits = CondomLimits::default();
    let interface: Box<dyn Interface> = Box::new(TestInterface);
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/test_compare.wasm_add"
    ));

    let runtime_module = RuntimeModule::new(
        module,
        gas_costs.clone(),
        Compiler::SP,
        condom_limits.clone(),
    )
    .unwrap();

    match runtime_module.clone() {
        RuntimeModule::ASModule(_) => {
            panic!("Must be WasmV1Module");
        }
        RuntimeModule::WasmV1Module(_) => {
            println!("Module type WasmV1Module");
        }
    }
    run_main(
        &*interface,
        runtime_module,
        100_000_000,
        gas_costs,
        condom_limits,
    )
    .unwrap();
}

#[test]
#[serial]
/// Test basic function-only SC execution
fn test_run_function() {
    let gas_costs = GasCosts::default();
    let condom_limits = CondomLimits::default();
    let interface: Box<dyn Interface> = Box::new(TestInterface);
    let module = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/wasm/basic_func.wasm"));

    let runtime_module = RuntimeModule::new(
        module,
        gas_costs.clone(),
        Compiler::SP,
        condom_limits.clone(),
    )
    .unwrap();
    run_function(
        &*interface,
        runtime_module,
        "ping",
        b"",
        100_000,
        gas_costs,
        condom_limits,
    )
    .unwrap();
}

// NOTE: this test is outdated as module are now pre-compiled with max_instance_cost
// leaving this for documentation purposes
// #[test]
// #[serial]
// /// Test both cases of the not enough gas error
// fn test_not_enough_gas_error() {
//     let gas_costs = GasCosts::default();
//     let interface: Box<dyn Interface> = Box::new(TestInterface);
//     let module = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/wasm/basic_main.wasm"));

//     // Test giving not enough gas to create the instance
//     let runtime_module = RuntimeModule::new(module, 100, gas_costs.clone(), Compiler::SP, condom_limits.clone()).unwrap();
//     let error = run_main(&*interface, runtime_module, 100_000, gas_costs.clone())
//         .unwrap_err()
//         .to_string();
//     assert!(
//         error.contains("Not enough gas, limit reached at initialization")
//             || error.contains("RuntimeError: unreachable")
//     );

//     // Test giving enough gas to create the instance but not enough for the VM
//     let runtime_module =
//         RuntimeModule::new(module, 100_000, gas_costs.clone(), Compiler::SP, condom_limits.clone()).unwrap();
//     let error = run_main(&*interface, runtime_module, 100, gas_costs)
//         .unwrap_err()
//         .to_string();
//     assert!(error.contains("Not enough gas to launch the virtual machine"));
// }

#[test]
#[serial]
/// Test that a no-main SC executed through `run_main` fails as expected
fn test_run_main_without_main() {
    let gas_costs = GasCosts::default();
    let condom_limits = CondomLimits::default();
    let interface: Box<dyn Interface> = Box::new(TestInterface);
    let module = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/wasm/no_main.wasm"));
    let runtime_module = RuntimeModule::new(
        module,
        gas_costs.clone(),
        Compiler::SP,
        condom_limits.clone(),
    )
    .unwrap();
    run_main(
        &*interface,
        runtime_module,
        100_000,
        gas_costs,
        condom_limits,
    )
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
    let condom_limits = CondomLimits::default();
    let interface: Box<dyn Interface> = Box::new(TestInterface);
    let module = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/wasm/empty_main.wasm"));
    gas_costs.launch_cost = 0;
    let runtime_module = RuntimeModule::new(
        module,
        gas_costs.clone(),
        Compiler::SP,
        condom_limits.clone(),
    )
    .unwrap();
    let a = run_main(
        &*interface,
        runtime_module.clone(),
        10_000_000,
        gas_costs.clone(),
        condom_limits.clone(),
    )
    .expect("Failed to run empty_main.wasm");
    // Here we avoid hard-coding a value (that can change in future wasmer
    // release)
    assert!(a.remaining_gas > 0);

    let mut rng = rand::thread_rng();
    let cost = rng.gen_range(1..1_000_000);
    gas_costs.launch_cost = cost;
    let b = run_main(
        &*interface,
        runtime_module,
        10_000_000,
        gas_costs,
        condom_limits,
    )
    .expect("Failed to run empty_main.wasm");
    // Between 2 calls, the metering cost should be the difference
    assert_eq!(a.remaining_gas - b.remaining_gas, cost);
}

#[test]
#[serial]
/// Even if our SC is empty there is still an initial and minimum metering cost,
/// because we have a memory allocator to init.
///
/// This test ensure that this initial cost is correctly debited.
fn test_run_main_rust_wasmv1() {
    // let mut gas_costs = GasCosts::default();
    // let interface: Box<dyn Interface> =
    //     Box::new(TestInterface);
    // let module = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"),
    //     "/../massa-rust-sc-examples/target/wasm32-unknown-unknown/debug/
    // massa_rust_sc_deploy_sc.wasm_add")); gas_costs.launch_cost = 0;
    // let runtime_module =
    //     RuntimeModule::new(module, gas_costs.clone(), Compiler::SP, condom_limits.clone())?;
    //         .unwrap();

    // let a = match run_main(
    //     &*interface,
    //     runtime_module.clone(),
    //     10_000_000,
    //     gas_costs.clone(),
    // ) { Ok(a) => a, Err(e) => { println!("e: {}", e); panic!("Failed to run
    //   main") }
    // };

    // // Here we avoid hard-coding a value (that can change in future wasmer
    // // release)
    // assert!(a.remaining_gas > 0);

    // let mut rng = rand::thread_rng();
    // let cost = rng.gen_range(1..1_000_000);
    // gas_costs.launch_cost = cost;
    // let b = run_main(&*interface, runtime_module, 10_000_000, gas_costs)
    //     .expect("Failed to run empty_main.wasm");
    // dbg!(b.ret);
    // // Between 2 calls, the metering cost should be the difference
    // // assert_eq!(a.remaining_gas - b.remaining_gas, cost);
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
    let condom_limits = CondomLimits::default();
    let interface: Box<dyn Interface> = Box::new(TestInterface);
    let module = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/wasm/op_fn.wasm"));
    let runtime_module = RuntimeModule::new(
        module,
        gas_costs.clone(),
        Compiler::SP,
        condom_limits.clone(),
    )
    .unwrap();
    run_main(
        &*interface,
        runtime_module,
        10_000_000,
        gas_costs.clone(),
        condom_limits.clone(),
    )
    .expect("Failed to run op_fn.wasm");
}

/// Test `seed`, `Date.now`, `console.log` and `abort`
///
/// These are AS functions that we choose to handle in the VM
#[test]
#[serial]
fn test_builtins() {
    let gas_costs = GasCosts::default();
    let condom_limits = CondomLimits::default();
    let interface: Box<dyn Interface> = Box::new(TestInterface);
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/use_builtins.wasm"
    ));
    let runtime_module = RuntimeModule::new(
        module,
        gas_costs.clone(),
        Compiler::SP,
        condom_limits.clone(),
    )
    .unwrap();
    // let before = chrono::offset::Utc::now().timestamp_millis();
    match run_main(
        &*interface,
        runtime_module,
        10_000_000,
        gas_costs.clone(),
        condom_limits.clone(),
    ) {
        Err(e) => {
            let msg = e.to_string();
            // make sure the error was caused by a manual abort
            assert!(msg.contains("Manual abort"), "Error was: {:?}", e);
            assert!(msg.contains("at use_builtins.ts"), "Error was: {:?}", e);
            // check the given timestamp validity
            // let after = chrono::offset::Utc::now().timestamp_millis();
            // let ident = "UTC timestamp (ms) = ";
            // let start =
            // msg.find(ident).unwrap_or(0).saturating_add(ident.len());
            // let end = msg.find(" at use_builtins.ts").unwrap_or(0);
            // let sc_timestamp: i64 = msg[start..end].parse().unwrap();
            // assert!(before <= sc_timestamp && sc_timestamp <= after);
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
    let condom_limits = CondomLimits::default();
    let interface: Box<dyn Interface> = Box::new(TestInterface);
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/use_builtin_assert.wasm"
    ));

    let runtime_module = RuntimeModule::new(
        module,
        gas_costs.clone(),
        Compiler::SP,
        condom_limits.clone(),
    )
    .unwrap();
    match run_function(
        &*interface,
        runtime_module,
        "assert_with_msg",
        b"",
        100_000,
        gas_costs.clone(),
        condom_limits.clone(),
    ) {
        Err(e) => {
            assert!(e.to_string().contains("Result is not true!"))
        }
        _ => panic!("test should return an error!"),
    }

    let runtime_module = RuntimeModule::new(
        module,
        gas_costs.clone(),
        Compiler::SP,
        condom_limits.clone(),
    )
    .unwrap();
    if run_function(
        &*interface,
        runtime_module,
        "assert_no_msg",
        b"",
        100_000,
        gas_costs.clone(),
        condom_limits.clone(),
    )
    .is_ok()
    {
        panic!("test should return an error!");
    }

    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/use_builtin_exit.wasm"
    ));

    let runtime_module = RuntimeModule::new(
        module,
        gas_costs.clone(),
        Compiler::SP,
        condom_limits.clone(),
    )
    .unwrap();
    match run_function(
        &*interface,
        runtime_module,
        "exit_no_code",
        b"",
        100_000,
        gas_costs.clone(),
        condom_limits.clone(),
    ) {
        Err(e) => {
            assert!(e.to_string().contains("exit with code: 0"))
        }
        _ => panic!("test should return an error!"),
    }

    let runtime_module = RuntimeModule::new(
        module,
        gas_costs.clone(),
        Compiler::SP,
        condom_limits.clone(),
    )
    .unwrap();
    match run_function(
        &*interface,
        runtime_module,
        "exit_with_code",
        b"",
        100_000,
        gas_costs,
        condom_limits,
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
    let condom_limits = CondomLimits::default();
    let interface: Box<dyn Interface> = Box::new(TestInterface);

    // Test for hrtime
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/unsupported_builtin_hrtime.wasm"
    ));
    let runtime_module = RuntimeModule::new(
        module,
        gas_costs.clone(),
        Compiler::SP,
        condom_limits.clone(),
    )
    .unwrap();

    match run_main(
        &*interface,
        runtime_module,
        10_000_000,
        gas_costs.clone(),
        condom_limits.clone(),
    ) {
        Err(e) => {
            assert!(e
                .to_string()
                .contains("Error while importing \"env\".\"performance.now\""))
        }
        _ => panic!("test should return an error!"),
    }

    // test for getRandomValues
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/unsupported_builtin_random_values.wasm"
    ));
    let runtime_module = RuntimeModule::new(
        module,
        gas_costs.clone(),
        Compiler::SP,
        condom_limits.clone(),
    )
    .unwrap();

    match run_main(
        &*interface,
        runtime_module,
        10_000_000,
        gas_costs.clone(),
        condom_limits.clone(),
    ) {
        Err(e) => {
            assert!(e
                .to_string()
                .contains("Error while importing \"env\".\"crypto.getRandomValuesN\""))
        }
        _ => panic!("test should return an error!"),
    }
}

#[test]
#[serial]
/// Ensure that WAT files (text equivalent of WASM) are rejected but their WASM
/// equivalent are accepted
///
/// WAT files are mostly used in testing
fn test_wat() {
    {
        let gas_costs = GasCosts::default();
        let condom_limits = CondomLimits::default();
        let bytecode = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/wasm/dummy.wat"));

        let runtime_module = RuntimeModule::new(
            bytecode,
            gas_costs.clone(),
            Compiler::SP,
            condom_limits.clone(),
        );

        match runtime_module {
            Ok(_) => panic!(".wat are not supported anymore"),
            Err(err) => {
                assert!(err.to_string().contains("Unsupported file format for SC"));
            }
        }
    }
    {
        let gas_costs = GasCosts::default();
        let condom_limits = CondomLimits::default();
        let interface: Box<dyn Interface> = Box::new(TestInterface);
        let bytecode = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/wasm/dummy.wasm"));

        let runtime_module = RuntimeModule::new(
            bytecode,
            gas_costs.clone(),
            Compiler::SP,
            condom_limits.clone(),
        )
        .unwrap();
        let response = run_main(
            &*interface,
            runtime_module,
            100_000,
            gas_costs.clone(),
            condom_limits.clone(),
        )
        .unwrap();

        // Note: for now, exec main always return an empty vec
        let excepted: Vec<u8> = Vec::new();
        assert_eq!(response.ret, excepted);
    }
}

#[test]
#[serial]
/// Test a WASM execution using features disabled in engine (simd & threads)
fn test_features_disabled() {
    let gas_costs = GasCosts::default();
    let condom_limits = CondomLimits::default();

    let module = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/wasm/simd.wasm"));
    match RuntimeModule::new(
        module,
        gas_costs.clone(),
        Compiler::SP,
        condom_limits.clone(),
    ) {
        Err(e) => {
            // println!("Error: {}", e);
            assert!(e
                .to_string()
                .starts_with("VM instance error: Validation error: SIMD support is not enabled"));
        }
        _ => panic!("Failed to run use_builtins.wasm"),
    }

    let module = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/wasm/threads.wasm"));
    match RuntimeModule::new(
        module,
        gas_costs.clone(),
        Compiler::SP,
        condom_limits.clone(),
    ) {
        Err(e) => {
            // println!("Error: {}", e);
            assert!(e.to_string().starts_with(
                "VM instance error: Validation error: threads support is not enabled"
            ));
        }
        _ => panic!("Failed to run use_builtins.wasm"),
    }
}

#[test]
#[serial]
/// Non regression test on the AS class id values
fn test_class_id() {
    // setup basic AS runtime context
    let interface: Box<dyn Interface> = Box::new(TestInterface);
    let bytecode = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/return_basic.wasm"
    ));
    let module = ASModule::new(
        bytecode,
        100_000,
        GasCosts::default(),
        Compiler::SP,
        CondomLimits::default(),
    )
    .unwrap();
    let mut store = Store::new(module._engine);
    let mut context = ASContext::new(
        &*interface,
        module.binary_module,
        GasCosts::default(),
        CondomLimits::default(),
    );
    let (instance, _function_env, _) = context.create_vm_instance_and_init_env(&mut store).unwrap();

    // setup test specific context
    let (_, fenv) = context.resolver(&mut store);

    // get string and array offsets
    let return_string = instance.exports.get_function("return_string").unwrap();
    let return_array = instance.exports.get_function("return_array").unwrap();
    let string_ptr = return_string
        .call(&mut store, &[])
        .unwrap()
        .first()
        .unwrap()
        .i32()
        .unwrap();
    let array_ptr = return_array
        .call(&mut store, &[])
        .unwrap()
        .first()
        .unwrap()
        .i32()
        .unwrap();

    let memory = context.env.get_ffi_env().memory.as_ref().unwrap();
    let fenv_mut = fenv.into_mut(&mut store);
    let memory_view = memory.view(&fenv_mut);

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

#[test]
#[serial]
fn test_abort_does_not_panic() {
    // AS source code for reference
    // ```ts
    //import { u256 } from 'as-bignum/assembly';
    //const test = u256.from<string>('1982982');
    //export function main(_args: StaticArray<u8>): void {
    //}
    // ```
    // The above code fails in the start method, causing wasmer to abort the
    // initialization of the module, hence returning memory as None that caused
    // a panic

    let interface: Box<dyn Interface> = Box::new(TestInterface);
    let bytecode = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/abort_crash.wasm"
    ));

    let module = ASModule::new(
        bytecode,
        100_000,
        GasCosts::default(),
        Compiler::SP,
        CondomLimits::default(),
    )
    .unwrap();
    let mut store = Store::new(module._engine);
    let mut context = ASContext::new(
        &*interface,
        module.binary_module,
        GasCosts::default(),
        CondomLimits::default(),
    );
    let create_vm_instance_and_init_env = context.create_vm_instance_and_init_env(&mut store);
    match create_vm_instance_and_init_env {
        Ok(_) => {
            panic!("Expected an error")
        }
        Err(e) => {
            assert_eq!(
                e.to_string(),
                "VM instance error: RuntimeError: Runtime error: AssemblyScript memory is missing from the environment"
            )
        }
    }
}

#[cfg(feature = "execution-trace")]
#[test]
#[serial]
/// Non regression test on the AS class id values
fn test_ser() {
    let at0 = AbiTraceType::Bool(true);
    let at1 = AbiTraceType::String("hello".to_string());
    let at2 = AbiTraceType::U64(33);
    let at3 = AbiTraceType::ByteArray(vec![1, 255, 0]);
    let at4 = AbiTraceType::ByteArrays(vec![vec![1, 255, 0], vec![11, 42, 33]]);
    let at5 = AbiTraceType::Strings(vec!["yo".to_string(), "yo2".to_string()]);
    let at6 = AbiTraceType::Slot((111, 22));
    let s0 = serde_json::to_string(&at0).unwrap();
    println!("s0: {}", s0);
    assert!(s0.find("bool").is_some());
    let s1 = serde_json::to_string(&at1).unwrap();
    println!("s1: {}", s1);
    assert!(s1.find("string").is_some());
    let s2 = serde_json::to_string(&at2).unwrap();
    println!("s2: {}", s2);
    assert!(s2.find("u64").is_some());
    let s3 = serde_json::to_string(&at3).unwrap();
    println!("s3: {}", s3);
    assert!(s3.find("byteArray").is_some());
    let s4 = serde_json::to_string(&at4).unwrap();
    println!("s4: {}", s4);
    assert!(s4.find("byteArrays").is_some());
    let s5 = serde_json::to_string(&at5).unwrap();
    println!("s5: {}", s5);
    assert!(s5.find("strings").is_some());
    let s6 = serde_json::to_string(&at6).unwrap();
    println!("s6: {}", s6);
    assert!(s6.find("slot").is_some());

    let atv1 = AbiTraceValue {
        name: "foo".to_string(),
        value: at6,
    };
    let s_atv1_ = serde_json::to_string(&atv1);
    println!("s_atv1: {:?}", s_atv1_);

    let s_atv1 = s_atv1_.unwrap();
    assert!(s_atv1.find("foo").is_some());
    assert!(s_atv1.find("slot").is_some());
}
