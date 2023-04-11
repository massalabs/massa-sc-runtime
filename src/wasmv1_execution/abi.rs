//! *abi_impl.rs* contains all the implementation (and some tools as
//! abi_bail!) of the massa abi.
//!
//! The ABIs are the imported function / object declared in the webassembly
//! module. You can look at the other side of the mirror in `massa.ts` and the
//! rust side in `execution_impl.rs`.

use as_ffi_bindings::{BufferPtr, Read as ASRead, StringPtr, Write as ASWrite};
use function_name::named;
use std::ops::Add;
use wasmer::{AsStoreMut, AsStoreRef, FunctionEnvMut, Memory, WasmPtr};

use super::env::{get_remaining_points, sub_remaining_gas_abi, WasmV1Env};
use crate::settings;

use super::common::{call_module, create_sc, function_exists, local_call};
use super::error::{abi_bail, ABIResult};


/// Retrieves the AssemblyScript environment.
///
/// Fails during instantiation to avoid gas manipulation in the WASM start function.
pub(crate) fn get_env(ctx: &FunctionEnvMut<WasmV1Env>) -> ABIResult<WasmV1Env> {
    let env = ctx.data().clone();
    if !(env.abi_enabled.load(std::sync::atomic::Ordering::Relaxed)) {
        abi_bail!("ABI calls are not available during instantiation");
    } else {
        Ok(env)
    }
}

fn read_buffer(memory: &Memory, ctx: &FunctionEnvMut<WasmV1Env>, ptr: i32) -> ABIResult<Vec<u8>> {
    let env = get_env(ctx)?;
    let mut buffer = Vec::new();
    let mut ptr: WasmPtr<u8> = WasmPtr::new(ptr as u32);

    let mut memory = memory.view::<u8>();



    let mut read = ASRead::new(&mut memory, &mut ptr);
    read.read_to_end(&mut buffer)?;
    Ok(buffer)
}

/// Transfer an amount from the address on the current call stack to a target address.
#[named]
pub(crate) fn abi_transfer_coins(
    mut ctx: FunctionEnvMut<WasmV1Env>,
    params: i32,
) -> ABIResult<i32> {
    let env = get_env(&ctx)?;
    
    sub_remaining_gas_abi(&env, &mut ctx, function_name!())?;
    if raw_amount.is_negative() {
        abi_bail!("Negative raw amount.");
    }
    let memory = get_memory!(env);
    let to_address = &read_string(memory, &ctx, to_address)?;
    // Do not remove this. It could be used for gas_calibration in future.
    // if cfg!(feature = "gas_calibration") {
    //     let fname = format!("massa.{}:0", function_name!());
    //     param_size_update(&env, &mut ctx, &fname, to_address.len(), true);
    // }
    Ok(env
        .get_interface()
        .transfer_coins(to_address, raw_amount as u64)?)
}

/// Raw call that have the right type signature to be able to be call a module
/// directly form AssemblyScript:
#[named]
pub(crate) fn abi_call(
    mut ctx: FunctionEnvMut<WasmV1Env>,
    address: i32,
    function: i32,
    param: i32,
    call_coins: i64,
) -> ABIResult<i32> {
    let env = get_env(&ctx)?;
    sub_remaining_gas_abi(&env, &mut ctx, function_name!())?;
    let memory = get_memory!(env);
    let address = &read_string(memory, &ctx, address)?;
    let function = &read_string(memory, &ctx, function)?;
    let param = &read_buffer(memory, &ctx, param)?;

    // Do not remove this. It could be used for gas_calibration in future.
    // if cfg!(feature = "gas_calibration") {
    //     let fname = format!("massa.{}:0", function_name!());
    //     param_size_update(&env, &mut ctx, &fname, address.len(), true);
    //     let fname = format!("massa.{}:1", function_name!());
    //     param_size_update(&env, &mut ctx, &fname, function.len(), true);
    //     let fname = format!("massa.{}:2", function_name!());
    //     param_size_update(&env, &mut ctx, &fname, param.len(), true);
    // }

    let response = call_module(&mut ctx, address, function, param, call_coins)?;
    match BufferPtr::alloc(&response.ret, env.get_ffi_env(), &mut ctx) {
        Ok(ret) => Ok(ret.offset() as i32),
        _ => abi_bail!(format!(
            "Cannot allocate response in call {}::{}",
            address, function
        )),
    }
}

/// sets a key-indexed data entry in the datastore, overwriting existing values if any
#[named]
pub(crate) fn abi_set_data(
    mut ctx: FunctionEnvMut<WasmV1Env>,
    key: i32,
    value: i32,
) -> ABIResult<()> {
    let env = get_env(&ctx)?;
    sub_remaining_gas_abi(&env, &mut ctx, function_name!())?;
    let memory = get_memory!(env);
    let key = read_buffer(memory, &ctx, key)?;
    let value = read_buffer(memory, &ctx, value)?;

    // Do not remove this. It could be used for gas_calibration in future.
    // if cfg!(feature = "gas_calibration") {
    //     param_size_update(
    //         &env,
    //         &mut ctx,
    //         "massa.assembly_script_set_data:0",
    //         key.len(),
    //         false,
    //     );
    //     param_size_update(
    //         &env,
    //         &mut ctx,
    //         "massa.assembly_script_set_data:1",
    //         value.len(),
    //         false,
    //     );
    // }

    env.get_interface().raw_set_data(&key, &value)?;
    Ok(())
}


/// gets a key-indexed data entry in the datastore, failing if non-existent
#[named]
pub(crate) fn abi_get_data(mut ctx: FunctionEnvMut<WasmV1Env>, key: i32) -> ABIResult<i32> {
    let env = get_env(&ctx)?;
    sub_remaining_gas_abi(&env, &mut ctx, function_name!())?;
    let memory = get_memory!(env);
    let key = read_buffer(memory, &ctx, key)?;
    // Do not remove this. It could be used for gas_calibration in future.
    // if cfg!(feature = "gas_calibration") {
    //     let fname = format!("massa.{}:0", function_name!());
    //     param_size_update(&env, &mut ctx, &fname, key.len(), true);
    // }
    let data = env.get_interface().raw_get_data(&key)?;
    Ok(pointer_from_bytearray(&env, &mut ctx, &data)?.offset() as i32)
}


/// Assembly script builtin `abort` function.
///
/// It prints the origin filename, an error messag, the line and column.
pub fn abi_abort(
    ctx: FunctionEnvMut<WasmV1Env>,
    message: StringPtr,
    filename: StringPtr,
    line: i32,
    col: i32,
) -> ABIResult<()> {
    let memory = ctx
        .data()
        .get_ffi_env()
        .memory
        .as_ref()
        .expect("Failed to get memory on env")
        .clone();
    let message_ = message
        .read(&memory, &ctx)
        .map_err(|e| wasmer::RuntimeError::new(e.to_string()));
    let filename_ = filename
        .read(&memory, &ctx)
        .map_err(|e| wasmer::RuntimeError::new(e.to_string()));

    if message_.is_err() || filename_.is_err() {
        abi_bail!("aborting failed to load message or filename")
    }
    abi_bail!(format!(
        "error: {} at {}:{} col: {}",
        message_.unwrap(),
        filename_.unwrap(),
        line,
        col
    ));
}


#[cfg(test)]
mod tests {
    use crate::wasmv1_execution::abi::ser_bytearray_vec;

    #[test]
    fn test_ser() {
        let vb: Vec<Vec<u8>> = vec![vec![1, 2, 3], vec![255]];

        let vb_ser = ser_bytearray_vec(&vb, vb.len(), 10).unwrap();
        assert_eq!(vb_ser, [2, 0, 0, 0, 3, 1, 2, 3, 1, 255]);
    }

    #[test]
    fn test_ser_edge_cases() {
        // FIXME: should we support theses edge cases or bail?

        let vb: Vec<Vec<u8>> = vec![vec![1, 2, 3], vec![]];

        let vb_ser = ser_bytearray_vec(&vb, vb.len(), 10).unwrap();
        assert_eq!(vb_ser, [2, 0, 0, 0, 3, 1, 2, 3, 0]);

        let vb_ser = ser_bytearray_vec(&vb, vb.len(), 1);
        assert!(vb_ser.is_err());

        let vb: Vec<Vec<u8>> = vec![];
        let vb_ser = ser_bytearray_vec(&vb, vb.len(), 10).unwrap();
        let empty_vec: Vec<u8> = vec![];
        assert_eq!(vb_ser, empty_vec);

        // A really big vec to serialize
        let vb: Vec<Vec<u8>> = (0..=u8::MAX)
            .cycle()
            .take(u16::MAX as usize)
            .map(|i| vec![i])
            .collect();
        assert_eq!(vb.len(), u16::MAX as usize);

        let vb_ser = ser_bytearray_vec(&vb, vb.len(), u16::MAX as usize).unwrap();
        assert_eq!(vb_ser[0..4], [255, 255, 0, 0]);
        assert_eq!(vb_ser[4], 1);
        assert_eq!(vb_ser[4 + 1], 0);
        assert_eq!(vb_ser[4 + 2], 1);
        assert_eq!(vb_ser[4 + 3], 1);
        assert_eq!(vb_ser[vb_ser.len() - 2], 1);
        assert_eq!(vb_ser[vb_ser.len() - 1], 254);
    }
}
