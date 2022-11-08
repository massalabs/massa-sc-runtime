use crate::{
    settings,
    types::Interface,
};
use crate::run_main_gc;
use crate::tests::{TestInterface, Ledger};
use crate::middlewares::gas_calibration::get_gas_calibration_result;
use crate::execution::{create_instance, get_module};

use serial_test::serial;
use anyhow::Result;
use parking_lot::Mutex;

use std::sync::Arc;

#[test]
#[serial]
fn test_basic_abi_call_counter() -> Result<()> {

    settings::reset_metering();
    let interface: TestInterface =
       TestInterface(Arc::new(Mutex::new(Ledger::new())));
    let bytecode = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/build/gc_abi_call_basic.wasm"
    ));

    let gas_calibration_result = run_main_gc(bytecode, 100000, &interface)?;

    println!("gas_calibration_result: {:?}", gas_calibration_result);
    // Total
    println!("Total: {}", gas_calibration_result.0.iter().fold(0, |acc, (_, v)| acc + v));
    assert_eq!(gas_calibration_result.0.len(), 2);
    // assert_eq!(gas_calibration_result.0.get("Abi:call:massa.assembly_script_print"), Some(&2));
    // assert_eq!(gas_calibration_result.0.get("Abi:call:env.abort"), Some(&0));

    Ok(())
}
