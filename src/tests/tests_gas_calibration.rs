use crate::execution::Compiler;
use crate::middlewares::operator::{OPERATOR_CARDINALITY, OPERATOR_VARIANTS};
use crate::middlewares::operator::{
    _OPERATOR_BULK_MEMORY, _OPERATOR_NON_TRAPPING_FLOAT_TO_INT, _OPERATOR_THREAD, _OPERATOR_VECTOR,
};
use crate::tests::TestInterface;
use crate::{run_main_gc, types::Interface, CondomLimits, GasCosts, RuntimeModule};
use std::collections::HashSet;

use anyhow::Result;
use more_asserts as ma;
use serial_test::serial;

#[test]
#[serial]
fn test_basic_abi_call_counter() -> Result<()> {
    let interface = TestInterface;
    let bytecode = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/gc_abi_call_basic.wasm"
    ));

    let gas_costs = GasCosts::default();
    let condom_limits = CondomLimits::default();
    let runtime_module = RuntimeModule::new(
        bytecode,
        gas_costs.clone(),
        Compiler::SP,
        condom_limits.clone(),
    )?;
    let gas_calibration_result = run_main_gc(
        &interface,
        runtime_module,
        b"",
        100_000,
        gas_costs.clone(),
        condom_limits.clone(),
    )?;
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
fn test_basic_abi_call_counter_wasmv1() -> Result<()> {
    let interface: Box<dyn Interface> = Box::new(TestInterface);
    let module = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/test_gc_abi_call_basic.wasm_add"
    ));

    let gas_costs = GasCosts::default();
    let condom_limits = CondomLimits::default();
    // let runtime_module =
    //     RuntimeModule::new(bytecode, 100_000, gas_costs.clone(),
    // Compiler::SP, CondomLimits::default())?;?;

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

    let gas_calibration_result = run_main_gc(
        &*interface,
        runtime_module,
        b"",
        100_000,
        gas_costs.clone(),
        condom_limits.clone(),
    )?;
    // println!("gas_calibration_result: {:?}", gas_calibration_result);

    // Note:
    // 2 counters for abi call count (generate_event / abort)
    // 2 counters for abi_generate_event (1 param) & abi_abort (1 param) param
    // size + counters for each operators
    assert_eq!(
        gas_calibration_result.counters.len(),
        // 2 + 5 + OPERATOR_CARDINALITY
        2 + 2 + OPERATOR_CARDINALITY
    );
    assert_eq!(
        gas_calibration_result
            .counters
            .get("Abi:call:massa.abi_generate_event"),
        Some(&2)
    );
    assert_eq!(
        gas_calibration_result
            .counters
            .get("Abi:call:massa.abi_abort"),
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
    let interface = TestInterface;
    let bytecode = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/gc_abi_call_for.wasm"
    ));

    let gas_costs = GasCosts::default();
    let condom_limits = CondomLimits::default();
    let runtime_module = RuntimeModule::new(
        bytecode,
        gas_costs.clone(),
        Compiler::SP,
        condom_limits.clone(),
    )?;
    let gas_calibration_result = run_main_gc(
        &interface,
        runtime_module,
        b"",
        100_000,
        gas_costs.clone(),
        condom_limits.clone(),
    )?;
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
fn test_basic_abi_call_loop_wasmv1() -> Result<()> {
    let interface = TestInterface;
    let bytecode = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/test_gc_abi_call_for.wasm_add"
    ));

    let gas_costs = GasCosts::default();
    let condom_limits = CondomLimits::default();
    let runtime_module = RuntimeModule::new(
        bytecode,
        gas_costs.clone(),
        Compiler::SP,
        condom_limits.clone(),
    )?;
    let gas_calibration_result = run_main_gc(
        &interface,
        runtime_module,
        b"",
        100_000,
        gas_costs.clone(),
        condom_limits.clone(),
    )?;
    assert_eq!(
        gas_calibration_result.counters.len(),
        2 + 2 + OPERATOR_CARDINALITY
    );
    assert_eq!(
        gas_calibration_result
            .counters
            .get("Abi:call:massa.abi_generate_event"),
        Some(&11)
    );
    assert_eq!(
        gas_calibration_result
            .counters
            .get("Abi:call:massa.abi_abort"),
        Some(&0)
    );

    Ok(())
}

#[test]
#[serial]
fn test_basic_op() -> Result<()> {
    let interface = TestInterface;
    let bytecode = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/gc_basic_op.wasm"
    ));

    let gas_costs = GasCosts::default();
    let condom_limits = CondomLimits::default();
    let runtime_module = RuntimeModule::new(
        bytecode,
        gas_costs.clone(),
        Compiler::SP,
        condom_limits.clone(),
    )?;
    let gas_calibration_result = run_main_gc(
        &interface,
        runtime_module,
        b"",
        100_000,
        gas_costs.clone(),
        condom_limits.clone(),
    )?;
    // 1 for env.abort + 4 env.abort parameters
    assert_eq!(
        gas_calibration_result.counters.len(),
        1 + 4 + OPERATOR_CARDINALITY
    );
    // Abi call issued
    // assert_eq!(gas_calibration_result.0.get("Abi:call:massa.
    // assembly_script_print"), Some(&1));
    assert_eq!(
        gas_calibration_result.counters.get("Abi:call:env.abort"),
        Some(&0)
    );

    // check op count
    // Use wat file to view op (https://webassembly.github.io/wabt/demo/wasm2wat/)
    let op_executed = HashSet::from([
        "Wasm:I32Add",
        "Wasm:I32GtU",
        "Wasm:End",
        "Wasm:I32Sub",
        "Wasm:I32Store",
        "Wasm:LocalTee",
        "Wasm:LocalGet",
        "Wasm:I32Const",
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
fn test_basic_op_wasmv1() -> Result<()> {
    let interface = TestInterface;
    let bytecode = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/test_gc_basic_op.wasm_add"
    ));

    let gas_costs = GasCosts::default();
    let condom_limits = CondomLimits::default();
    let runtime_module = RuntimeModule::new(
        bytecode,
        gas_costs.clone(),
        Compiler::SP,
        condom_limits.clone(),
    )?;
    let gas_calibration_result = run_main_gc(
        &interface,
        runtime_module,
        b"",
        100_000,
        gas_costs.clone(),
        condom_limits.clone(),
    )?;
    // 1 for env.abort + 1 env.abort parameters
    assert_eq!(
        gas_calibration_result.counters.len(),
        1 + 1 + OPERATOR_CARDINALITY
    );
    // Abi call issued
    assert_eq!(
        gas_calibration_result
            .counters
            .get("Abi:call:massa.abi_abort"),
        Some(&0)
    );

    // check op count
    // Use wat file to view op (https://webassembly.github.io/wabt/demo/wasm2wat/)
    let op_executed = HashSet::from([
        "Wasm:I32Add",
        "Wasm:I32GtU",
        "Wasm:End",
        "Wasm:I32Sub",
        "Wasm:I32Store",
        "Wasm:LocalTee",
        "Wasm:LocalGet",
        "Wasm:I32Const",
    ]);

    for op_exec in &op_executed {
        ma::assert_gt!(gas_calibration_result.counters.get(*op_exec).unwrap(), &0);
    }

    // Note: op_executed is not exhaustive and the SC has a lot of code included
    //       so for now the following check is disabled

    // for (k, v) in gas_calibration_result.counters.iter() {
    // if (*k).starts_with("Wasm:") && !op_executed.contains(&((*k).as_str()))
    // {
    // println!("Wasm op: {:?}", k);
    // assert_eq!(*v, 0);
    // }
    // }

    Ok(())
}

#[test]
#[ignore]
#[serial]
fn test_basic_abi_call_param_size() -> Result<()> {
    let interface = TestInterface;
    let bytecode = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/gc_abi_call_param_size.wasm"
    ));

    let gas_costs = GasCosts::default();
    let condom_limits = CondomLimits::default();
    let runtime_module = RuntimeModule::new(
        bytecode,
        gas_costs.clone(),
        Compiler::SP,
        condom_limits.clone(),
    )?;
    let gas_calibration_result = run_main_gc(
        &interface,
        runtime_module,
        b"9876543",
        100_000,
        gas_costs.clone(),
        condom_limits.clone(),
    )?;
    // println!("gas_calibration_result: {:?}", gas_calibration_result);

    // Note:
    // 3 counters for abi call count (assembly_script_print +
    // assembly_script_set_data + abort) 7 counters for
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

    // For now, this has been disabled in the code, so we disabled it here too
    // assert_eq!(
    // gas_calibration_result
    // .counters
    // .get("Abi:ps:massa.assembly_script_set_data:0"),
    // Some(&8)
    // );
    // assert_eq!(
    // gas_calibration_result
    // .counters
    // .get("Abi:ps:massa.assembly_script_set_data:1"),
    // Some(&20)
    // );
    //
    // Check param len send via run_main_gc + 2x (utf-16)
    // TODO / FIXME: should be 14 but is now 18 - because param is now passed as
    // &[u8] instead of &str assert_eq!(
    // gas_calibration_result
    // .counters
    // .get("Abi:ps:massa.assembly_script_print:0"),
    // Some(&18)
    // );

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
    // Check that OPERATOR_* are ~ "valid"
    // OPERATOR_* arrays are defined manually or using some python scripts so we
    // need to ensure that everything defined in ok
    // Here we assume that OPERATOR_VARIANTS is valid (e.g. contains all wasm op
    // name)

    let op_variants = HashSet::from(OPERATOR_VARIANTS);
    assert_eq!(op_variants.len(), OPERATOR_VARIANTS.len());

    assert_eq!(
        HashSet::from(_OPERATOR_THREAD).len(),
        _OPERATOR_THREAD.len()
    );
    assert_eq!(
        HashSet::from(_OPERATOR_VECTOR).len(),
        _OPERATOR_VECTOR.len()
    );
    assert_eq!(
        HashSet::from(_OPERATOR_BULK_MEMORY).len(),
        _OPERATOR_BULK_MEMORY.len()
    );
    assert_eq!(
        HashSet::from(_OPERATOR_NON_TRAPPING_FLOAT_TO_INT).len(),
        _OPERATOR_NON_TRAPPING_FLOAT_TO_INT.len()
    );

    let op_iterator = _OPERATOR_THREAD
        .iter()
        .chain(_OPERATOR_VECTOR.iter())
        .chain(_OPERATOR_BULK_MEMORY.iter())
        .chain(_OPERATOR_NON_TRAPPING_FLOAT_TO_INT.iter());

    for operator_name in op_iterator {
        println!("Checking operator name: {}", operator_name);
        assert!(op_variants.contains(operator_name))
    }
}
