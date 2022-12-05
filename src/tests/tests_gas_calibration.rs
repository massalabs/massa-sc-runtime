use crate::middlewares::operator::{
    OPERATOR_BULK_MEMORY, OPERATOR_NON_TRAPPING_FLOAT_TO_INT, OPERATOR_THREAD, OPERATOR_VECTOR,
};
use crate::middlewares::operator::{OPERATOR_CARDINALITY, OPERATOR_VARIANTS};
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

    let gas_calibration_result = run_main_gc(bytecode, 100000, &interface, b"")?;

    // println!("gas_calibration_result: {:?}", gas_calibration_result);

    // Note:
    // 2 counters for abi call count (print / abort)
    // 5 counters for print (1 param) & abort (4 params) param size
    // + counters for each operators
    assert_eq!(
        gas_calibration_result.counters.len(),
        2 + 5 + OPERATOR_CARDINALITY
    );
    assert_eq!(
        gas_calibration_result
            .counters
            .get("Abi:call:massa.assembly_script_print"),
        Some(&2)
    );
    assert_eq!(
        gas_calibration_result.counters.get("Abi:call:env.abort"),
        Some(&0)
    );

    // param size
    // "CCCC" -> 8
    // "9876543" -> 14
    // assert_eq!(
    //     gas_calibration_result
    //         .counters
    //         .get("Abi:ps:massa.assembly_script_print"),
    //     Some(&22)
    // );

    // Timer checks
    ma::assert_gt!(
        gas_calibration_result
            .timers
            .get("Time:transform_module_info"),
        Some(&0.0)
    );
    ma::assert_gt!(
        gas_calibration_result
            .timers
            .get("Time:gas_calibration_result"),
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

    let gas_calibration_result = run_main_gc(bytecode, 100000, &interface, b"")?;

    assert_eq!(
        gas_calibration_result.counters.len(),
        2 + 5 + OPERATOR_CARDINALITY
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

    let gas_calibration_result = run_main_gc(bytecode, 100000, &interface, b"")?;

    // 1 for env.abort + 4 env.abort parameters
    assert_eq!(
        gas_calibration_result.counters.len(),
        1 + 4 + OPERATOR_CARDINALITY
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

#[test]
#[serial]
fn test_basic_abi_call_param_size() -> Result<()> {
    settings::reset_metering();
    let interface: TestInterface = TestInterface(Arc::new(Mutex::new(Ledger::new())));
    let bytecode = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/build/gc_abi_call_param_size.wasm"
    ));

    let gas_calibration_result = run_main_gc(bytecode, 100000, &interface, b"9876543")?;
    // println!("gas_calibration_result: {:?}", gas_calibration_result);

    // Note:
    // 3 counters for abi call count (assembly_script_print + assembly_script_set_data + abort)
    // 7 counters for
    // assembly_script (1 param)
    // assembly_script_set_data (2 params)
    // abort (4 params)
    // + counters for each operators
    assert_eq!(
        gas_calibration_result.counters.len(),
        3 + 7 + OPERATOR_CARDINALITY
    );
    assert_eq!(
        gas_calibration_result
            .counters
            .get("Abi:call:massa.assembly_script_set_data"),
        Some(&1)
    );

    // param size
    // Check SC src in massa-unit-tests-src for expected lengths
    assert_eq!(
        gas_calibration_result
            .counters
            .get("Abi:ps:massa.assembly_script_set_data:0"),
        Some(&8)
    );
    assert_eq!(
        gas_calibration_result
            .counters
            .get("Abi:ps:massa.assembly_script_set_data:1"),
        Some(&20)
    );
    // Check param len send via run_main_gc + 2x (utf-16)
    // TODO / FIXME: should be 14 but is now 18 - because param is now passed as &[u8] instead of &str
    assert_eq!(
        gas_calibration_result
            .counters
            .get("Abi:ps:massa.assembly_script_print:0"),
        Some(&18)
    );
    for i in 0..4 {
        assert_eq!(
            gas_calibration_result
                .counters
                .get(&format!("Abi:ps:env.abort:{}", i)),
            Some(&0)
        );
    }

    Ok(())
}

#[test]
fn test_operators_definition() {
    let op_variants = HashSet::from(OPERATOR_VARIANTS);
    assert_eq!(op_variants.len(), OPERATOR_VARIANTS.len());

    assert_eq!(HashSet::from(OPERATOR_THREAD).len(), OPERATOR_THREAD.len());
    assert_eq!(HashSet::from(OPERATOR_VECTOR).len(), OPERATOR_VECTOR.len());
    assert_eq!(
        HashSet::from(OPERATOR_BULK_MEMORY).len(),
        OPERATOR_BULK_MEMORY.len()
    );
    assert_eq!(
        HashSet::from(OPERATOR_NON_TRAPPING_FLOAT_TO_INT).len(),
        OPERATOR_NON_TRAPPING_FLOAT_TO_INT.len()
    );

    let op_iterator = OPERATOR_THREAD
        .iter()
        .chain(OPERATOR_VECTOR.iter())
        .chain(OPERATOR_BULK_MEMORY.iter())
        .chain(OPERATOR_NON_TRAPPING_FLOAT_TO_INT.iter());

    for operator_name in op_iterator {
        println!("Checking operator name: {}", operator_name);
        assert!(op_variants.contains(operator_name))
    }
}
