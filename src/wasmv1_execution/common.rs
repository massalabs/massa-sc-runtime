//! Execution functions used by the ABIs.
//!
//! IMPORTANT: these were designed for, and should not be called outside of ABIs.

/*

use wasmer::FunctionEnvMut;
use wasmer_middlewares::metering::get_remaining_points;

use super::{env::WasmV1ABIEnv, WasmV1Error};
use crate::Response;



/// Alternative to `call_module` to execute bytecode in a local context
pub(crate) fn local_call(
    ctx: &mut FunctionEnvMut<WasmV1ABIEnv>,
    bytecode: &[u8],
    function: &str,
    param: &[u8],
) -> Result<Response, WasmV1Error> {
    let env = get_env(ctx)?;
    let interface = env.get_interface();

    let remaining_gas = if cfg!(feature = "gas_calibration") {
        u64::MAX
    } else {
        get_remaining_points(&env, ctx)?
    };

    let module = interface.get_module(bytecode, remaining_gas)?;
    let resp = crate::execution::run_function(
        &*interface,
        module,
        function,
        param,
        remaining_gas,
        env.get_gas_costs(),
    )?;
    if cfg!(not(feature = "gas_calibration")) {
        set_remaining_points(&env, ctx, resp.remaining_gas)?;
    }
    Ok(resp)
}
*/
