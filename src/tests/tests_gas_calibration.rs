use crate::{
    settings,
    types::Interface,
};
use crate::execution_impl::exec_gc;
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
    let interface: Box<dyn Interface> =
        Box::new(TestInterface(Arc::new(Mutex::new(Ledger::new()))));
    let bytecode = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/build/gc_abi_call_basic.wasm"
    ));

    let module = get_module(&*interface, bytecode).unwrap();
    let instance = create_instance(10_000_000, &module).unwrap();
    let (_response, instance) = exec_gc(instance, module, settings::MAIN, "")?;
    let gas_calibration_result = get_gas_calibration_result(&instance);

    assert_eq!(gas_calibration_result.0.len(), 2);
    assert_eq!(gas_calibration_result.0.get("Abi:call:massa.assembly_script_print"), Some(&2));
    assert_eq!(gas_calibration_result.0.get("Abi:call:env.abort"), Some(&0));

    Ok(())
}

#[test]
#[serial]
fn test_basic_abi_call_loop() -> Result<()> {

    settings::reset_metering();
    let interface: Box<dyn Interface> =
        Box::new(TestInterface(Arc::new(Mutex::new(Ledger::new()))));
    let bytecode = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wasm/build/gc_abi_call_for.wasm"
    ));

    let module = get_module(&*interface, bytecode).unwrap();
    let instance = create_instance(10_000_000, &module).unwrap();
    let (_response, instance) = exec_gc(instance, module, settings::MAIN, "")?;
    let gas_calibration_result = get_gas_calibration_result(&instance);

    assert_eq!(gas_calibration_result.0.len(), 2);
    assert_eq!(gas_calibration_result.0.get("Abi:call:massa.assembly_script_print"), Some(&11));
    assert_eq!(gas_calibration_result.0.get("Abi:call:env.abort"), Some(&0));

    Ok(())
}
