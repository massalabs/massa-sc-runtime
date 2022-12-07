use crate::middlewares::operator::OPERATOR_CARDINALITY;
use crate::run_main_gc;
use crate::settings;
use crate::tests::{Ledger, TestInterface};
use std::collections::HashSet;

use anyhow::Result;
use more_asserts as ma;
use parking_lot::Mutex;
use serial_test::serial;

use std::sync::Arc;

#[test]
#[serial]
fn test_basic_abi_call_counter() -> Result<()> {
    settings::reset_metering();
    let interface: TestInterface = TestInterface(Arc::new(Mutex::new(Ledger::new())));
    let bytecode = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/build/gc_abi_call_basic.wasm"
    ));

    let gas_calibration_result = run_main_gc(bytecode, 100000, &interface)?;

    // println!("gas_calibration_result: {:?}", gas_calibration_result);

    assert_eq!(
        gas_calibration_result.counters.len(),
        2 + OPERATOR_CARDINALITY
    );
    assert_eq!(
        gas_calibration_result
            .counters
            .get("Abi:call:massa.assembly_script_print"),
        Some(&4)
    );
    assert_eq!(
        gas_calibration_result.counters.get("Abi:call:env.abort"),
        Some(&0)
    );

    ma::assert_gt!(
        gas_calibration_result
            .timers
            .get("Time:transform_module_info"),
        Some(&0.0)
    );

    Ok(())
}

#[test]
#[serial]
fn test_basic_abi_call_loop() -> Result<()> {
    settings::reset_metering();
    let interface: TestInterface = TestInterface(Arc::new(Mutex::new(Ledger::new())));
    let bytecode = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/build/gc_abi_call_for.wasm"
    ));

    let gas_calibration_result = run_main_gc(bytecode, 100000, &interface)?;

    assert_eq!(
        gas_calibration_result.counters.len(),
        2 + OPERATOR_CARDINALITY
    );
    assert_eq!(
        gas_calibration_result
            .counters
            .get("Abi:call:massa.assembly_script_print"),
        Some(&11)
    );
    assert_eq!(
        gas_calibration_result.counters.get("Abi:call:env.abort"),
        Some(&0)
    );

    Ok(())
}

#[test]
#[serial]
fn test_basic_op() -> Result<()> {
    settings::reset_metering();
    let interface: TestInterface = TestInterface(Arc::new(Mutex::new(Ledger::new())));
    let bytecode = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/build/gc_op_basic.wasm"
    ));

    let gas_calibration_result = run_main_gc(bytecode, 100000, &interface)?;

    assert_eq!(
        gas_calibration_result.counters.len(),
        1 + OPERATOR_CARDINALITY
    );
    // Abi call issued
    // assert_eq!(gas_calibration_result.0.get("Abi:call:massa.assembly_script_print"), Some(&1));
    assert_eq!(
        gas_calibration_result.counters.get("Abi:call:env.abort"),
        Some(&0)
    );

    // check op count
    // Use wat file to view op (https://webassembly.github.io/wabt/demo/wasm2wat/)
    let op_executed = HashSet::from([
        "Wasm:I32Add",
        "Wasm:I32And",
        "Wasm:I32GtU",
        "Wasm:End",
        "Wasm:I32Sub",
        "Wasm:I32Shl",
        "Wasm:I32Store",
        "Wasm:GlobalSet",
        "Wasm:LocalTee",
        "Wasm:LocalGet",
        "Wasm:GlobalGet",
        "Wasm:I32Const",
        // "Wasm:If",
        "Wasm:MemorySize",
    ]);

    for op_exec in &op_executed {
        ma::assert_gt!(gas_calibration_result.counters.get(*op_exec).unwrap(), &0);
    }

    for (k, v) in gas_calibration_result.counters.iter() {
        if (*k).starts_with("Wasm:") && !op_executed.contains(&((*k).as_str())) {
            assert_eq!(*v, 0);
        }
    }

    Ok(())
}
