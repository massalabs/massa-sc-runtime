//! *abi_impl.rs* contains all the implementation (and some tools as
//! abi_bail!) of the massa abi.
//!
//! The ABIs are the imported function / object declared in the webassembly
//! module. You can look at the other side of the mirror in `massa.ts` and the
//! rust side in `execution_impl.rs`.

use as_ffi_bindings::{BufferPtr, Read as ASRead, StringPtr, Write as ASWrite};
use std::ops::Add;
use wasmer::{AsStoreRef, FunctionEnvMut, Memory};

use super::env::{sub_remaining_gas_with_globals, ASEnv};
use crate::{as_execution::ABIError, settings};

/// Optimized macro for simple ABI calls - single ctx.data() call
macro_rules! abi {
    ($ctx:expr, $gas_field:ident, $body:expr) => {{
        let (gas_cost, remaining, exhausted) = {
            let data = $ctx.data();
            if !data.abi_enabled.load(std::sync::atomic::Ordering::Relaxed) {
                abi_bail!("ABI calls are not available during instantiation");
            }
            let gas = data.get_gas_costs().$gas_field;
            let rem = match data.remaining_points.as_ref() {
                Some(g) => g.clone(),
                None => abi_bail!("Lost remaining_points"),
            };
            let exh = match data.exhausted_points.as_ref() {
                Some(g) => g.clone(),
                None => abi_bail!("Lost exhausted_points"),
            };
            (gas, rem, exh)
        };
        sub_remaining_gas_with_globals(&remaining, &exhausted, &mut $ctx, gas_cost)?;
        $body
    }};
}

/// Optimized macro for ABIs needing memory - extracts memory in same ctx.data() call
/// Usage: abi_with_memory!(ctx, gas_field, |memory| { body with memory })
macro_rules! abi_with_memory {
    ($ctx:expr, $gas_field:ident, |$memory:ident| $body:expr) => {{
        let (gas_cost, remaining, exhausted, $memory) = {
            let data = $ctx.data();
            if !data.abi_enabled.load(std::sync::atomic::Ordering::Relaxed) {
                abi_bail!("ABI calls are not available during instantiation");
            }
            let gas = data.get_gas_costs().$gas_field;
            let rem = match data.remaining_points.as_ref() {
                Some(g) => g.clone(),
                None => abi_bail!("Lost remaining_points"),
            };
            let exh = match data.exhausted_points.as_ref() {
                Some(g) => g.clone(),
                None => abi_bail!("Lost exhausted_points"),
            };
            let mem = match data.get_ffi_env().memory.as_ref() {
                Some(m) => m.clone(),
                None => abi_bail!("Failed to get memory"),
            };
            (gas, rem, exh, mem)
        };
        sub_remaining_gas_with_globals(&remaining, &exhausted, &mut $ctx, gas_cost)?;
        $body
    }};
}

#[cfg(feature = "execution-trace")]
use crate::{
    into_trace_value,
    types::{AbiTrace, AbiTraceType},
};

use super::common::{call_module, create_sc, function_exists, local_call};
use super::error::{abi_bail, ABIResult};

macro_rules! get_memory {
    ($env:ident) => {
        match $env.get_ffi_env().memory.as_ref() {
            Some(mem) => mem,
            _ => abi_bail!("AssemblyScript memory is missing from the environment"),
        }
    };
}

/// Retrieves the AssemblyScript environment.
///
/// Fails during instantiation to avoid gas manipulation in the WASM start
/// function.
pub(crate) fn get_env(ctx: &FunctionEnvMut<ASEnv>) -> ABIResult<ASEnv> {
    let env = ctx.data().clone();
    if !(env.abi_enabled.load(std::sync::atomic::Ordering::Relaxed)) {
        abi_bail!("ABI calls are not available during instantiation");
    } else {
        Ok(env)
    }
}

/// Get the coins that have been made available for a specific purpose for the
/// current call.
pub(crate) fn assembly_script_get_call_coins(mut ctx: FunctionEnvMut<ASEnv>) -> ABIResult<i64> {
    abi!(ctx, assembly_script_get_call_coins, {
        let res = ctx.data().interface.get_call_coins()? as i64;
        #[cfg(feature = "execution-trace")]
        ctx.data_mut().trace.push(AbiTrace {
            name: "assembly_script_get_call_coins".to_string(),
            params: vec![],
            return_value: res.into(),
            sub_calls: None,
        });
        Ok(res)
    })
}

/// Transfer an amount from the address on the current call stack to a target
/// address.
pub(crate) fn assembly_script_transfer_coins(
    mut ctx: FunctionEnvMut<ASEnv>,
    to_address: i32,
    raw_amount: i64,
) -> ABIResult<()> {
    if raw_amount.is_negative() {
        abi_bail!("Negative raw amount.");
    }
    abi_with_memory!(ctx, assembly_script_transfer_coins, |memory| {
        let to_address = read_string(&memory, &ctx, to_address)?;
        ctx.data()
            .interface
            .transfer_coins(&to_address, raw_amount as u64)?;
        #[cfg(feature = "execution-trace")]
        {
            let call_stack = ctx.data().interface.get_call_stack();
            let from_address = call_stack
                .unwrap_or_default()
                .last()
                .cloned()
                .unwrap_or_else(|| "".to_string());

            ctx.data_mut().trace.push(AbiTrace {
                name: "assembly_script_transfer_coins".to_string(),
                params: vec![
                    into_trace_value!(from_address),
                    into_trace_value!(to_address),
                    (stringify!(raw_amount), raw_amount as u64).into(),
                ],
                return_value: AbiTraceType::None,
                sub_calls: None,
            });
        }
        Ok(())
    })
}

/// Transfer an amount from the specified address to a target address.
pub(crate) fn assembly_script_transfer_coins_for(
    mut ctx: FunctionEnvMut<ASEnv>,
    from_address: i32,
    to_address: i32,
    raw_amount: i64,
) -> ABIResult<()> {
    if raw_amount.is_negative() {
        abi_bail!("Negative raw amount.");
    }
    abi_with_memory!(ctx, assembly_script_transfer_coins_for, |memory| {
        let from_address = read_string(&memory, &ctx, from_address)?;
        let to_address = read_string(&memory, &ctx, to_address)?;
        ctx.data()
            .interface
            .transfer_coins_for(&from_address, &to_address, raw_amount as u64)?;
        #[cfg(feature = "execution-trace")]
        ctx.data_mut().trace.push(AbiTrace {
            name: "assembly_script_transfer_coins_for".to_string(),
            params: vec![
                into_trace_value!(from_address),
                into_trace_value!(to_address),
                (stringify!(raw_amount), raw_amount as u64).into(),
            ],
            return_value: AbiTraceType::None,
            sub_calls: None,
        });
        Ok(())
    })
}

pub(crate) fn assembly_script_get_balance(mut ctx: FunctionEnvMut<ASEnv>) -> ABIResult<i64> {
    abi!(ctx, assembly_script_get_balance, {
        let res = ctx.data().interface.get_balance()? as i64;
        #[cfg(feature = "execution-trace")]
        ctx.data_mut().trace.push(AbiTrace {
            name: "assembly_script_get_balance".to_string(),
            params: vec![],
            return_value: res.into(),
            sub_calls: None,
        });
        Ok(res)
    })
}

pub(crate) fn assembly_script_get_balance_for(
    mut ctx: FunctionEnvMut<ASEnv>,
    address: i32,
) -> ABIResult<i64> {
    abi_with_memory!(ctx, assembly_script_get_balance_for, |memory| {
        let address = read_string(&memory, &ctx, address)?;
        let res = ctx.data().interface.get_balance_for(&address)? as i64;
        #[cfg(feature = "execution-trace")]
        ctx.data_mut().trace.push(AbiTrace {
            name: "assembly_script_get_balance_for".to_string(),
            params: vec![into_trace_value!(address)],
            return_value: res.into(),
            sub_calls: None,
        });
        Ok(res)
    })
}

/// Raw call that have the right type signature to be able to be call a module
/// directly form AssemblyScript:
pub(crate) fn assembly_script_call(
    mut ctx: FunctionEnvMut<ASEnv>,
    address: i32,
    function: i32,
    param: i32,
    call_coins: i64,
) -> ABIResult<i32> {
    abi_with_memory!(ctx, assembly_script_call, |memory| {
        let address = read_string(&memory, &ctx, address)?;
        let function = read_string(&memory, &ctx, function)?;
        let param = read_buffer(&memory, &ctx, param)?;

        let response = call_module(&mut ctx, &address, &function, &param, call_coins)?;
        #[cfg(feature = "execution-trace")]
        ctx.data_mut().trace.push(AbiTrace {
            name: "assembly_script_call".to_string(),
            params: vec![
                ("address", address.clone()).into(),
                ("function", function.clone()).into(),
                into_trace_value!(param),
                into_trace_value!(call_coins),
            ],
            return_value: response.ret.clone().into(),
            sub_calls: Some(response.trace),
        });
        let ffi_env = ctx.data().get_ffi_env().clone();
        match BufferPtr::alloc(&response.ret, &ffi_env, &mut ctx) {
            Ok(ret) => Ok(ret.offset() as i32),
            _ => abi_bail!(format!(
                "Cannot allocate response in call {}::{}",
                address, function
            )),
        }
    })
}

pub(crate) fn assembly_script_get_remaining_gas(mut ctx: FunctionEnvMut<ASEnv>) -> ABIResult<i64> {
    // Inline macro logic: check enabled, get gas cost, extract globals, subtract gas
    let (gas_cost, remaining, exhausted) = {
        let data = ctx.data();
        if !data.abi_enabled.load(std::sync::atomic::Ordering::Relaxed) {
            abi_bail!("ABI calls are not available during instantiation");
        }
        let gas = data.get_gas_costs().assembly_script_get_remaining_gas;
        let rem = match data.remaining_points.as_ref() {
            Some(g) => g.clone(),
            None => abi_bail!("Lost remaining_points"),
        };
        let exh = match data.exhausted_points.as_ref() {
            Some(g) => g.clone(),
            None => abi_bail!("Lost exhausted_points"),
        };
        (gas, rem, exh)
    };
    sub_remaining_gas_with_globals(&remaining, &exhausted, &mut ctx, gas_cost)?;

    // Now read the remaining gas after subtraction (same logic as get_remaining_points)
    if cfg!(feature = "gas_calibration") {
        #[cfg(feature = "execution-trace")]
        ctx.data_mut().trace.push(AbiTrace {
            name: "assembly_script_get_remaining_gas".to_string(),
            params: vec![],
            return_value: (u64::MAX as i64).into(),
            sub_calls: None,
        });
        return Ok(u64::MAX as i64);
    }

    // Check if exhausted
    match exhausted.get(&mut ctx).try_into() {
        Ok::<i32, _>(exhausted_val) if exhausted_val > 0 => {
            #[cfg(feature = "execution-trace")]
            ctx.data_mut().trace.push(AbiTrace {
                name: "assembly_script_get_remaining_gas".to_string(),
                params: vec![],
                return_value: 0i64.into(),
                sub_calls: None,
            });
            return Ok(0);
        }
        Ok::<i32, _>(_) => (),
        Err(_) => abi_bail!("exhausted_points has wrong type"),
    }

    // Read remaining gas
    let res = match remaining.get(&mut ctx).try_into() {
        Ok::<u64, _>(remaining_val) => remaining_val as i64,
        Err(_) => abi_bail!("remaining_points has wrong type"),
    };

    #[cfg(feature = "execution-trace")]
    ctx.data_mut().trace.push(AbiTrace {
        name: "assembly_script_get_remaining_gas".to_string(),
        params: vec![],
        return_value: res.into(),
        sub_calls: None,
    });
    Ok(res)
}

/// Create an instance of VM from a module with a
/// given interface, an operation number limit and a webassembly module
///
/// An utility print function to write on stdout directly from AssemblyScript:
pub(crate) fn assembly_script_print(mut ctx: FunctionEnvMut<ASEnv>, arg: i32) -> ABIResult<()> {
    abi_with_memory!(ctx, assembly_script_print, |memory| {
        let message = read_string(&memory, &ctx, arg)?;
        ctx.data().interface.print(&message)?;
        #[cfg(feature = "execution-trace")]
        ctx.data_mut().trace.push(AbiTrace {
            name: "assembly_script_print".to_string(),
            params: vec![into_trace_value!(message)],
            return_value: AbiTraceType::None,
            sub_calls: None,
        });
        Ok(())
    })
}

/// Get the operation datastore keys (aka entries)
pub(crate) fn assembly_script_get_op_keys(mut ctx: FunctionEnvMut<ASEnv>) -> ABIResult<i32> {
    abi!(ctx, assembly_script_get_op_keys, {
        match ctx.data().interface.get_op_keys(None) {
            Err(err) => abi_bail!(err),
            Ok(keys) => {
                let fmt_keys =
                    ser_bytearray_vec(&keys, keys.len(), settings::max_op_datastore_entry_count())?;
                let ffi_env = ctx.data().get_ffi_env().clone();
                let ptr = BufferPtr::alloc(&fmt_keys, &ffi_env, &mut ctx)?.offset();

                #[cfg(feature = "execution-trace")]
                ctx.data_mut().trace.push(AbiTrace {
                    name: "assembly_script_get_op_keys".to_string(),
                    params: vec![],
                    return_value: fmt_keys.into(),
                    sub_calls: None,
                });
                Ok(ptr as i32)
            }
        }
    })
}

/// Get the operation datastore keys (aka entries)
pub(crate) fn assembly_script_get_op_keys_prefix(
    mut ctx: FunctionEnvMut<ASEnv>,
    prefix: i32,
) -> ABIResult<i32> {
    abi_with_memory!(ctx, assembly_script_get_op_keys_prefix, |memory| {
        let prefix = read_buffer(&memory, &ctx, prefix)?;
        let prefix_opt = if !prefix.is_empty() {
            Some(prefix.as_ref())
        } else {
            None
        };
        match ctx.data().interface.get_op_keys(prefix_opt) {
            Err(err) => abi_bail!(err),
            Ok(keys) => {
                let fmt_keys =
                    ser_bytearray_vec(&keys, keys.len(), settings::max_op_datastore_entry_count())?;
                let ffi_env = ctx.data().get_ffi_env().clone();
                let ptr = BufferPtr::alloc(&fmt_keys, &ffi_env, &mut ctx)?.offset();

                #[cfg(feature = "execution-trace")]
                ctx.data_mut().trace.push(AbiTrace {
                    name: "assembly_script_get_op_keys_prefix".to_string(),
                    params: vec![into_trace_value!(prefix)],
                    return_value: AbiTraceType::ByteArray(fmt_keys),
                    sub_calls: None,
                });
                Ok(ptr as i32)
            }
        }
    })
}

/// Check if a key is present in operation datastore
pub(crate) fn assembly_script_has_op_key(
    mut ctx: FunctionEnvMut<ASEnv>,
    key: i32,
) -> ABIResult<i32> {
    abi_with_memory!(ctx, assembly_script_has_op_key, |memory| {
        let key_bytes = read_buffer(&memory, &ctx, key)?;

        match ctx.data().interface.op_entry_exists(&key_bytes) {
            Err(err) => abi_bail!(err),
            Ok(b) => {
                // https://doc.rust-lang.org/reference/types/boolean.html
                // 'true' is explicitly defined as: 0x01 while 'false' is: 0x00
                let b_vec: Vec<u8> = vec![b as u8];
                let ffi_env = ctx.data().get_ffi_env().clone();
                let a = BufferPtr::alloc(&b_vec, &ffi_env, &mut ctx)?.offset();

                #[cfg(feature = "execution-trace")]
                ctx.data_mut().trace.push(AbiTrace {
                    name: "assembly_script_has_op_key".to_string(),
                    params: vec![into_trace_value!(key_bytes)],
                    return_value: b_vec.into(),
                    sub_calls: None,
                });

                Ok(a as i32)
            }
        }
    })
}

/// Get the operation datastore value associated to given key
pub(crate) fn assembly_script_get_op_data(
    mut ctx: FunctionEnvMut<ASEnv>,
    key: i32,
) -> ABIResult<i32> {
    abi_with_memory!(ctx, assembly_script_get_op_data, |memory| {
        let key_bytes = read_buffer(&memory, &ctx, key)?;

        let data = ctx.data().interface.get_op_data(&key_bytes)?;
        let ffi_env = ctx.data().get_ffi_env().clone();
        let ptr = BufferPtr::alloc(&data, &ffi_env, &mut ctx)?.offset() as i32;

        #[cfg(feature = "execution-trace")]
        ctx.data_mut().trace.push(AbiTrace {
            name: "assembly_script_get_op_data".to_string(),
            params: vec![into_trace_value!(key_bytes)],
            return_value: data.into(),
            sub_calls: None,
        });
        Ok(ptr)
    })
}

/// Read a bytecode string, representing the webassembly module binary encoded
/// with in base64.
pub(crate) fn assembly_script_create_sc(
    mut ctx: FunctionEnvMut<ASEnv>,
    bytecode: i32,
) -> ABIResult<i32> {
    abi_with_memory!(ctx, assembly_script_create_sc, |memory| {
        let bytecode: Vec<u8> = read_buffer(&memory, &ctx, bytecode)?;

        let address = create_sc(&mut ctx, &bytecode)?;
        let ffi_env = ctx.data().get_ffi_env().clone();
        let ptr = StringPtr::alloc(&address, &ffi_env, &mut ctx)?.offset() as i32;

        #[cfg(feature = "execution-trace")]
        ctx.data_mut().trace.push(AbiTrace {
            name: "assembly_script_create_sc".to_string(),
            params: vec![into_trace_value!(bytecode)],
            return_value: address.clone().into(),
            sub_calls: None,
        });
        Ok(ptr)
    })
}

/// performs a hash on a bytearray and returns the hash
pub(crate) fn assembly_script_hash(mut ctx: FunctionEnvMut<ASEnv>, value: i32) -> ABIResult<i32> {
    abi_with_memory!(ctx, assembly_script_hash, |memory| {
        let bytes = read_buffer(&memory, &ctx, value)?;

        let hash = ctx.data().interface.hash(&bytes)?.to_vec();
        let ffi_env = ctx.data().get_ffi_env().clone();
        let ptr = BufferPtr::alloc(&hash, &ffi_env, &mut ctx)?.offset();
        #[cfg(feature = "execution-trace")]
        ctx.data_mut().trace.push(AbiTrace {
            name: "assembly_script_hash".to_string(),
            params: vec![into_trace_value!(bytes)],
            return_value: hash.into(),
            sub_calls: None,
        });
        Ok(ptr as i32)
    })
}

/// performs a hash on a bytearray and returns the hash
pub(crate) fn assembly_script_keccak256_hash(
    mut ctx: FunctionEnvMut<ASEnv>,
    value: i32,
) -> ABIResult<i32> {
    abi_with_memory!(ctx, assembly_script_keccak256_hash, |memory| {
        let bytes = read_buffer(&memory, &ctx, value)?;
        let hash = ctx.data().interface.hash_keccak256(&bytes)?.to_vec();
        let ffi_env = ctx.data().get_ffi_env().clone();
        let ptr = BufferPtr::alloc(&hash, &ffi_env, &mut ctx)?.offset();

        #[cfg(feature = "execution-trace")]
        ctx.data_mut().trace.push(AbiTrace {
            name: "assembly_script_keccak".to_string(),
            params: vec![into_trace_value!(bytes)],
            return_value: hash.into(),
            sub_calls: None,
        });

        Ok(ptr as i32)
    })
}

/// Get keys (aka entries) in the datastore
pub(crate) fn assembly_script_get_keys(
    mut ctx: FunctionEnvMut<ASEnv>,
    prefix: i32,
) -> ABIResult<i32> {
    abi_with_memory!(ctx, assembly_script_get_keys, |memory| {
        let prefix = read_buffer(&memory, &ctx, prefix)?;
        let prefix_opt = if !prefix.is_empty() {
            Some(prefix.as_ref())
        } else {
            None
        };
        let keys = ctx.data().interface.get_keys(prefix_opt)?;
        let fmt_keys = ser_bytearray_vec(&keys, keys.len(), settings::max_datastore_entry_count())?;
        let ffi_env = ctx.data().get_ffi_env().clone();
        let ptr = BufferPtr::alloc(&fmt_keys, &ffi_env, &mut ctx)?.offset();

        #[cfg(feature = "execution-trace")]
        ctx.data_mut().trace.push(AbiTrace {
            name: "assembly_script_get_keys".to_string(),
            params: vec![into_trace_value!(prefix)],
            return_value: fmt_keys.into(),
            sub_calls: None,
        });
        Ok(ptr as i32)
    })
}

/// Get keys (aka entries) in the datastore
pub(crate) fn assembly_script_get_keys_for(
    mut ctx: FunctionEnvMut<ASEnv>,
    address: i32,
    prefix: i32,
) -> ABIResult<i32> {
    abi_with_memory!(ctx, assembly_script_get_keys_for, |memory| {
        let address = read_string(&memory, &ctx, address)?;
        let prefix = read_buffer(&memory, &ctx, prefix)?;
        let prefix_opt = if !prefix.is_empty() {
            Some(prefix.as_ref())
        } else {
            None
        };
        let keys = ctx.data().interface.get_keys_for(&address, prefix_opt)?;
        let fmt_keys = ser_bytearray_vec(&keys, keys.len(), settings::max_datastore_entry_count())?;
        let ffi_env = ctx.data().get_ffi_env().clone();
        let ptr = BufferPtr::alloc(&fmt_keys, &ffi_env, &mut ctx)?.offset();

        #[cfg(feature = "execution-trace")]
        ctx.data_mut().trace.push(AbiTrace {
            name: "assembly_script_get_keys_for".to_string(),
            params: vec![into_trace_value!(address), into_trace_value!(prefix)],
            return_value: AbiTraceType::ByteArrays(keys.iter().cloned().collect()),
            sub_calls: None,
        });
        Ok(ptr as i32)
    })
}

/// sets a key-indexed data entry in the datastore, overwriting existing values
/// if any
pub(crate) fn assembly_script_set_data(
    mut ctx: FunctionEnvMut<ASEnv>,
    key: i32,
    value: i32,
) -> ABIResult<()> {
    abi_with_memory!(ctx, assembly_script_set_data, |memory| {
        let key = read_buffer(&memory, &ctx, key)?;
        let value = read_buffer(&memory, &ctx, value)?;
        ctx.data().interface.raw_set_data(&key, &value)?;
        #[cfg(feature = "execution-trace")]
        ctx.data_mut().trace.push(AbiTrace {
            name: "assembly_script_set_data".to_string(),
            params: vec![into_trace_value!(key), into_trace_value!(value)],
            return_value: AbiTraceType::None,
            sub_calls: None,
        });
        Ok(())
    })
}

/// appends data to a key-indexed data entry in the datastore, fails if the
/// entry does not exist
pub(crate) fn assembly_script_append_data(
    mut ctx: FunctionEnvMut<ASEnv>,
    key: i32,
    value: i32,
) -> ABIResult<()> {
    abi_with_memory!(ctx, assembly_script_append_data, |memory| {
        let key = read_buffer(&memory, &ctx, key)?;
        let value = read_buffer(&memory, &ctx, value)?;
        ctx.data().interface.raw_append_data(&key, &value)?;
        #[cfg(feature = "execution-trace")]
        ctx.data_mut().trace.push(AbiTrace {
            name: "assembly_script_append_data".to_string(),
            params: vec![into_trace_value!(key), into_trace_value!(value)],
            return_value: AbiTraceType::None,
            sub_calls: None,
        });
        Ok(())
    })
}

/// gets a key-indexed data entry in the datastore, failing if non-existent
pub(crate) fn assembly_script_get_data(mut ctx: FunctionEnvMut<ASEnv>, key: i32) -> ABIResult<i32> {
    abi_with_memory!(ctx, assembly_script_get_data, |memory| {
        let key = read_buffer(&memory, &ctx, key)?;

        let data = ctx.data().interface.raw_get_data(&key)?;
        let ffi_env = ctx.data().get_ffi_env().clone();
        let ptr = BufferPtr::alloc(&data, &ffi_env, &mut ctx)?.offset() as i32;
        #[cfg(feature = "execution-trace")]
        ctx.data_mut().trace.push(AbiTrace {
            name: "assembly_script_get_data".to_string(),
            params: vec![into_trace_value!(key)],
            return_value: data.clone().into(),
            sub_calls: None,
        });
        Ok(ptr)
    })
}

/// checks if a key-indexed data entry exists in the datastore
pub(crate) fn assembly_script_has_data(mut ctx: FunctionEnvMut<ASEnv>, key: i32) -> ABIResult<i32> {
    abi_with_memory!(ctx, assembly_script_has_data, |memory| {
        let key = read_buffer(&memory, &ctx, key)?;
        let res = ctx.data().interface.has_data(&key)?;
        #[cfg(feature = "execution-trace")]
        ctx.data_mut().trace.push(AbiTrace {
            name: "assembly_script_has_data".to_string(),
            params: vec![into_trace_value!(key)],
            return_value: res.into(),
            sub_calls: None,
        });
        Ok(res as i32)
    })
}

/// deletes a key-indexed data entry in the datastore of the current address,
/// fails if the entry is absent
pub(crate) fn assembly_script_delete_data(
    mut ctx: FunctionEnvMut<ASEnv>,
    key: i32,
) -> ABIResult<()> {
    abi_with_memory!(ctx, assembly_script_delete_data, |memory| {
        let key = read_buffer(&memory, &ctx, key)?;
        ctx.data().interface.raw_delete_data(&key)?;
        #[cfg(feature = "execution-trace")]
        ctx.data_mut().trace.push(AbiTrace {
            name: "assembly_script_delete_data".to_string(),
            params: vec![into_trace_value!(key)],
            return_value: AbiTraceType::None,
            sub_calls: None,
        });
        Ok(())
    })
}

/// Sets the value of a datastore entry of an arbitrary address, creating the
/// entry if it does not exist. Fails if the address does not exist.
pub(crate) fn assembly_script_set_data_for(
    mut ctx: FunctionEnvMut<ASEnv>,
    address: i32,
    key: i32,
    value: i32,
) -> ABIResult<()> {
    abi_with_memory!(ctx, assembly_script_set_data_for, |memory| {
        let key = read_buffer(&memory, &ctx, key)?;
        let value = read_buffer(&memory, &ctx, value)?;
        let address = read_string(&memory, &ctx, address)?;
        ctx.data()
            .interface
            .raw_set_data_for(&address, &key, &value)?;
        #[cfg(feature = "execution-trace")]
        ctx.data_mut().trace.push(AbiTrace {
            name: "assembly_script_set_data_for".to_string(),
            params: vec![
                into_trace_value!(address),
                into_trace_value!(key),
                into_trace_value!(value),
            ],
            return_value: AbiTraceType::None,
            sub_calls: None,
        });
        Ok(())
    })
}

/// Appends data to the value of a datastore entry of an arbitrary address,
/// fails if the entry or address does not exist.
pub(crate) fn assembly_script_append_data_for(
    mut ctx: FunctionEnvMut<ASEnv>,
    address: i32,
    key: i32,
    value: i32,
) -> ABIResult<()> {
    abi_with_memory!(ctx, assembly_script_append_data_for, |memory| {
        let key = read_buffer(&memory, &ctx, key)?;
        let value = read_buffer(&memory, &ctx, value)?;
        let address = read_string(&memory, &ctx, address)?;
        ctx.data()
            .interface
            .raw_append_data_for(&address, &key, &value)?;
        #[cfg(feature = "execution-trace")]
        ctx.data_mut().trace.push(AbiTrace {
            name: "assembly_script_append_data_for".to_string(),
            params: vec![
                into_trace_value!(address),
                into_trace_value!(key),
                into_trace_value!(value),
            ],
            return_value: AbiTraceType::None,
            sub_calls: None,
        });
        Ok(())
    })
}

/// Gets the value of a datastore entry for an arbitrary address, fails if the
/// entry or address does not exist
pub(crate) fn assembly_script_get_data_for(
    mut ctx: FunctionEnvMut<ASEnv>,
    address: i32,
    key: i32,
) -> ABIResult<i32> {
    abi_with_memory!(ctx, assembly_script_get_data_for, |memory| {
        let address = read_string(&memory, &ctx, address)?;
        let key = read_buffer(&memory, &ctx, key)?;

        let data = ctx.data().interface.raw_get_data_for(&address, &key)?;
        let ffi_env = ctx.data().get_ffi_env().clone();
        let ptr = BufferPtr::alloc(&data, &ffi_env, &mut ctx)?.offset() as i32;
        #[cfg(feature = "execution-trace")]
        ctx.data_mut().trace.push(AbiTrace {
            name: "assembly_script_get_data_for".to_string(),
            params: vec![into_trace_value!(address), into_trace_value!(key)],
            return_value: data.into(),
            sub_calls: None,
        });
        Ok(ptr)
    })
}

/// Deletes a datastore entry for an address. Fails if the entry or address does
/// not exist.
pub(crate) fn assembly_script_delete_data_for(
    mut ctx: FunctionEnvMut<ASEnv>,
    address: i32,
    key: i32,
) -> ABIResult<()> {
    abi_with_memory!(ctx, assembly_script_delete_data_for, |memory| {
        let address = read_string(&memory, &ctx, address)?;
        let key = read_buffer(&memory, &ctx, key)?;
        ctx.data().interface.raw_delete_data_for(&address, &key)?;
        #[cfg(feature = "execution-trace")]
        ctx.data_mut().trace.push(AbiTrace {
            name: "assembly_script_delete_data_for".to_string(),
            params: vec![into_trace_value!(address), into_trace_value!(key)],
            return_value: AbiTraceType::None,
            sub_calls: None,
        });
        Ok(())
    })
}

pub(crate) fn assembly_script_has_data_for(
    mut ctx: FunctionEnvMut<ASEnv>,
    address: i32,
    key: i32,
) -> ABIResult<i32> {
    abi_with_memory!(ctx, assembly_script_has_data_for, |memory| {
        let address = read_string(&memory, &ctx, address)?;
        let key = read_buffer(&memory, &ctx, key)?;
        let res = ctx.data().interface.has_data_for(&address, &key)?;
        #[cfg(feature = "execution-trace")]
        ctx.data_mut().trace.push(AbiTrace {
            name: "assembly_script_has_data_for".to_string(),
            params: vec![into_trace_value!(address), into_trace_value!(key)],
            return_value: res.into(),
            sub_calls: None,
        });
        Ok(res as i32)
    })
}

pub(crate) fn assembly_script_get_owned_addresses(
    mut ctx: FunctionEnvMut<ASEnv>,
) -> ABIResult<i32> {
    abi!(ctx, assembly_script_get_owned_addresses, {
        let data = ctx.data().interface.get_owned_addresses()?;
        #[allow(clippy::let_and_return)]
        let ptr = alloc_string_array(&mut ctx, &data);
        #[cfg(feature = "execution-trace")]
        ctx.data_mut().trace.push(AbiTrace {
            name: "assembly_script_get_owned_addresses".to_string(),
            params: vec![],
            return_value: data.into(),
            sub_calls: None,
        });
        ptr
    })
}

pub(crate) fn assembly_script_get_call_stack(mut ctx: FunctionEnvMut<ASEnv>) -> ABIResult<i32> {
    abi!(ctx, assembly_script_get_call_stack, {
        let data = ctx.data().interface.get_call_stack()?;
        #[allow(clippy::let_and_return)]
        let ptr = alloc_string_array(&mut ctx, &data);
        #[cfg(feature = "execution-trace")]
        ctx.data_mut().trace.push(AbiTrace {
            name: "assembly_script_get_call_stack".to_string(),
            params: vec![],
            return_value: data.into(),
            sub_calls: None,
        });
        ptr
    })
}

pub(crate) fn assembly_script_generate_event(
    mut ctx: FunctionEnvMut<ASEnv>,
    event: i32,
) -> ABIResult<()> {
    abi_with_memory!(ctx, assembly_script_generate_event, |memory| {
        let event = read_string(&memory, &ctx, event)?;
        ctx.data().interface.generate_event(event.clone())?;
        #[cfg(feature = "execution-trace")]
        ctx.data_mut().trace.push(AbiTrace {
            name: "assembly_script_generate_event".to_string(),
            params: vec![into_trace_value!(event)],
            return_value: AbiTraceType::None,
            sub_calls: None,
        });
        Ok(())
    })
}

/// verify a signature of data given a public key. Returns Ok(1) if correctly
/// verified, otherwise Ok(0)
pub(crate) fn assembly_script_signature_verify(
    mut ctx: FunctionEnvMut<ASEnv>,
    data: i32,
    signature: i32,
    public_key: i32,
) -> ABIResult<i32> {
    abi_with_memory!(ctx, assembly_script_signature_verify, |memory| {
        let data = read_string(&memory, &ctx, data)?;
        let signature = read_string(&memory, &ctx, signature)?;
        let public_key = read_string(&memory, &ctx, public_key)?;
        let res =
            ctx.data()
                .interface
                .signature_verify(data.as_bytes(), &signature, &public_key)?;
        #[cfg(feature = "execution-trace")]
        ctx.data_mut().trace.push(AbiTrace {
            name: "assembly_script_signature_verify".to_string(),
            params: vec![
                into_trace_value!(data.as_bytes().to_vec()),
                into_trace_value!(signature),
                into_trace_value!(public_key),
            ],
            return_value: res.into(),
            sub_calls: None,
        });
        Ok(res as i32)
    })
}

/// Verify an EVM signature.
/// Returns Ok(1) if correctly verified, Ok(0) otherwise.
pub(crate) fn assembly_script_evm_signature_verify(
    mut ctx: FunctionEnvMut<ASEnv>,
    data: i32,
    signature: i32,
    public_key: i32,
) -> ABIResult<i32> {
    abi_with_memory!(ctx, assembly_script_evm_signature_verify, |memory| {
        let data = read_buffer(&memory, &ctx, data)?;
        let signature = read_buffer(&memory, &ctx, signature)?;
        let public_key = read_buffer(&memory, &ctx, public_key)?;
        let res = ctx
            .data()
            .interface
            .evm_signature_verify(&data, &signature, &public_key)?;
        #[cfg(feature = "execution-trace")]
        ctx.data_mut().trace.push(AbiTrace {
            name: "assembly_script_evm_signature_verify".to_string(),
            params: vec![
                into_trace_value!(data),
                into_trace_value!(signature),
                into_trace_value!(public_key),
            ],
            return_value: res.into(),
            sub_calls: None,
        });
        Ok(res as i32)
    })
}

/// Get address from public key (EVM)
pub(crate) fn assembly_script_evm_get_address_from_pubkey(
    mut ctx: FunctionEnvMut<ASEnv>,
    public_key: i32,
) -> ABIResult<i32> {
    abi_with_memory!(ctx, assembly_script_evm_get_address_from_pubkey, |memory| {
        let public_key = read_buffer(&memory, &ctx, public_key)?;
        let address = ctx
            .data()
            .interface
            .evm_get_address_from_pubkey(&public_key)?;
        let ffi_env = ctx.data().get_ffi_env().clone();
        let ptr = BufferPtr::alloc(&address, &ffi_env, &mut ctx)?.offset();
        #[cfg(feature = "execution-trace")]
        ctx.data_mut().trace.push(AbiTrace {
            name: "assembly_script_evm_get_address_from_pubkey".to_string(),
            params: vec![into_trace_value!(public_key)],
            return_value: address.into(),
            sub_calls: None,
        });
        Ok(ptr as i32)
    })
}

/// Get public key from signature (EVM)
pub(crate) fn assembly_script_evm_get_pubkey_from_signature(
    mut ctx: FunctionEnvMut<ASEnv>,
    data: i32,
    signature: i32,
) -> ABIResult<i32> {
    abi_with_memory!(
        ctx,
        assembly_script_evm_get_pubkey_from_signature,
        |memory| {
            let data = read_buffer(&memory, &ctx, data)?;
            let signature = read_buffer(&memory, &ctx, signature)?;
            let public_key = ctx
                .data()
                .interface
                .evm_get_pubkey_from_signature(&data, &signature)?;
            let ffi_env = ctx.data().get_ffi_env().clone();
            let ptr = BufferPtr::alloc(&public_key, &ffi_env, &mut ctx)?.offset();
            #[cfg(feature = "execution-trace")]
            ctx.data_mut().trace.push(AbiTrace {
                name: "assembly_script_evm_get_pubkey_from_signature".to_string(),
                params: vec![into_trace_value!(data), into_trace_value!(signature)],
                return_value: public_key.into(),
                sub_calls: None,
            });
            Ok(ptr as i32)
        }
    )
}

/// Return Ok(1) if the address is a User address, Ok(0) if it is an SC address
pub(crate) fn assembly_script_is_address_eoa(
    mut ctx: FunctionEnvMut<ASEnv>,
    address: i32,
) -> ABIResult<i32> {
    abi_with_memory!(ctx, assembly_script_is_address_eoa, |memory| {
        let address = read_string(&memory, &ctx, address)?;
        let res = ctx.data().interface.is_address_eoa(&address)?;
        #[cfg(feature = "execution-trace")]
        ctx.data_mut().trace.push(AbiTrace {
            name: "assembly_script_is_address_eoa".to_string(),
            params: vec![into_trace_value!(address)],
            return_value: res.into(),
            sub_calls: None,
        });
        Ok(res as i32)
    })
}

/// converts a public key to an address
pub(crate) fn assembly_script_address_from_public_key(
    mut ctx: FunctionEnvMut<ASEnv>,
    public_key: i32,
) -> ABIResult<i32> {
    abi_with_memory!(ctx, assembly_script_address_from_public_key, |memory| {
        let public_key = read_string(&memory, &ctx, public_key)?;

        let addr = ctx.data().interface.address_from_public_key(&public_key)?;
        let ffi_env = ctx.data().get_ffi_env().clone();
        let ptr = StringPtr::alloc(&addr, &ffi_env, &mut ctx)?.offset() as i32;
        #[cfg(feature = "execution-trace")]
        ctx.data_mut().trace.push(AbiTrace {
            name: "assembly_script_address_from_public_key".to_string(),
            params: vec![into_trace_value!(public_key)],
            return_value: addr.into(),
            sub_calls: None,
        });
        Ok(ptr)
    })
}

/// Validates an address is correct
pub(crate) fn assembly_script_validate_address(
    mut ctx: FunctionEnvMut<ASEnv>,
    address: i32,
) -> ABIResult<i32> {
    abi_with_memory!(ctx, assembly_script_validate_address, |memory| {
        let address = read_string(&memory, &ctx, address)?;
        let res = ctx.data().interface.validate_address(&address)?;
        #[cfg(feature = "execution-trace")]
        ctx.data_mut().trace.push(AbiTrace {
            name: "assembly_script_validate_address".to_string(),
            params: vec![into_trace_value!(address)],
            return_value: res.into(),
            sub_calls: None,
        });
        Ok(res as i32)
    })
}

/// generates an unsafe random number
pub(crate) fn assembly_script_unsafe_random(mut ctx: FunctionEnvMut<ASEnv>) -> ABIResult<i64> {
    abi!(ctx, assembly_script_unsafe_random, {
        let res = ctx.data().interface.unsafe_random()?;
        #[cfg(feature = "execution-trace")]
        ctx.data_mut().trace.push(AbiTrace {
            name: "assembly_script_unsafe_random".to_string(),
            params: vec![],
            return_value: res.into(),
            sub_calls: None,
        });
        Ok(res)
    })
}

/// gets the current unix timestamp in milliseconds
pub(crate) fn assembly_script_get_time(mut ctx: FunctionEnvMut<ASEnv>) -> ABIResult<i64> {
    abi!(ctx, assembly_script_get_time, {
        let res = ctx.data().interface.get_time()?;
        #[cfg(feature = "execution-trace")]
        ctx.data_mut().trace.push(AbiTrace {
            name: "assembly_script_get_time".to_string(),
            params: vec![],
            return_value: res.into(),
            sub_calls: None,
        });
        Ok(res as i64)
    })
}

/// sends an async message
#[allow(clippy::too_many_arguments)]
pub(crate) fn assembly_script_send_message(
    mut ctx: FunctionEnvMut<ASEnv>,
    target_address: i32,
    target_handler: i32,
    validity_start_period: i64,
    validity_start_thread: i32,
    validity_end_period: i64,
    validity_end_thread: i32,
    max_gas: i64,
    raw_fee: i64,
    raw_coins: i64,
    data: i32,
    filter_address: i32,
    filter_datastore_key: i32,
) -> ABIResult<()> {
    let validity_start: (u64, u8) = match (
        validity_start_period.try_into(),
        validity_start_thread.try_into(),
    ) {
        (Ok(p), Ok(t)) => (p, t),
        (Err(_), _) => abi_bail!("negative validity start period"),
        (_, Err(_)) => abi_bail!("invalid validity start thread"),
    };
    let validity_end: (u64, u8) = match (
        validity_end_period.try_into(),
        validity_end_thread.try_into(),
    ) {
        (Ok(p), Ok(t)) => (p, t),
        (Err(_), _) => abi_bail!("negative validity end period"),
        (_, Err(_)) => abi_bail!("invalid validity end thread"),
    };
    if max_gas.is_negative() {
        abi_bail!("negative max gas");
    }
    if raw_fee.is_negative() {
        abi_bail!("negative raw_fee");
    }
    if raw_coins.is_negative() {
        abi_bail!("negative coins")
    }

    abi_with_memory!(ctx, assembly_script_send_message, |memory| {
        let target_address = read_string(&memory, &ctx, target_address)?;
        let target_handler = read_string(&memory, &ctx, target_handler)?;
        let data = read_buffer(&memory, &ctx, data)?;
        let filter_address_string = read_string(&memory, &ctx, filter_address)?;
        let key = read_buffer(&memory, &ctx, filter_datastore_key)?;
        let filter = match (filter_address_string.as_str(), key.as_slice()) {
            ("", _) => None,
            (addr, &[]) => Some((addr, None)),
            (addr, key) => Some((addr, Some(key))),
        };

        ctx.data().interface.send_message(
            &target_address,
            &target_handler,
            validity_start,
            validity_end,
            max_gas as u64,
            raw_fee as u64,
            raw_coins as u64,
            &data,
            filter,
        )?;

        #[cfg(feature = "execution-trace")]
        ctx.data_mut().trace.push(AbiTrace {
            name: "assembly_script_send_message".to_string(),
            params: vec![
                into_trace_value!(target_address),
                into_trace_value!(target_handler),
                into_trace_value!(validity_start_period),
                into_trace_value!(validity_start_thread),
                into_trace_value!(validity_end_period),
                into_trace_value!(validity_end_thread),
                into_trace_value!(max_gas as u64),
                into_trace_value!(raw_fee as u64),
                into_trace_value!(raw_coins as u64),
                into_trace_value!(data),
                into_trace_value!(filter_address_string),
                into_trace_value!(key),
            ],
            return_value: AbiTraceType::None,
            sub_calls: None,
        });

        Ok(())
    })
}

/// converts a public key to an address
pub(crate) fn assembly_script_get_origin_operation_id(
    mut ctx: FunctionEnvMut<ASEnv>,
) -> ABIResult<i32> {
    abi!(ctx, assembly_script_get_origin_operation_id, {
        let operation_id = ctx
            .data()
            .interface
            .get_origin_operation_id()?
            .unwrap_or_default();
        let ffi_env = ctx.data().get_ffi_env().clone();
        let ptr = StringPtr::alloc(&operation_id, &ffi_env, &mut ctx)?.offset() as i32;
        #[cfg(feature = "execution-trace")]
        ctx.data_mut().trace.push(AbiTrace {
            name: "assembly_script_get_origin_operation_id".to_string(),
            params: vec![],
            return_value: operation_id.into(),
            sub_calls: None,
        });
        Ok(ptr)
    })
}

/// gets the period of the current execution slot
pub(crate) fn assembly_script_get_current_period(mut ctx: FunctionEnvMut<ASEnv>) -> ABIResult<i64> {
    abi!(ctx, assembly_script_get_current_period, {
        let current_period = ctx.data().interface.get_current_period()?;
        #[cfg(feature = "execution-trace")]
        ctx.data_mut().trace.push(AbiTrace {
            name: "assembly_script_get_current_period".to_string(),
            params: vec![],
            return_value: current_period.into(),
            sub_calls: None,
        });
        Ok(current_period as i64)
    })
}

/// gets the thread of the current execution slot
pub(crate) fn assembly_script_get_current_thread(mut ctx: FunctionEnvMut<ASEnv>) -> ABIResult<i32> {
    abi!(ctx, assembly_script_get_current_thread, {
        let current_thread = ctx.data().interface.get_current_thread()?;
        #[cfg(feature = "execution-trace")]
        ctx.data_mut().trace.push(AbiTrace {
            name: "assembly_script_get_current_thread".to_string(),
            params: vec![],
            return_value: current_thread.into(),
            sub_calls: None,
        });
        Ok(current_thread as i32)
    })
}

/// sets the executable bytecode of an arbitrary address
pub(crate) fn assembly_script_set_bytecode_for(
    mut ctx: FunctionEnvMut<ASEnv>,
    address: i32,
    bytecode: i32,
) -> ABIResult<()> {
    abi_with_memory!(ctx, assembly_script_set_bytecode_for, |memory| {
        let address = read_string(&memory, &ctx, address)?;
        let bytecode_raw = read_buffer(&memory, &ctx, bytecode)?;
        ctx.data()
            .interface
            .raw_set_bytecode_for(&address, &bytecode_raw)?;
        #[cfg(feature = "execution-trace")]
        ctx.data_mut().trace.push(AbiTrace {
            name: "assembly_script_set_bytecode_for".to_string(),
            params: vec![into_trace_value!(address), into_trace_value!(bytecode_raw)],
            return_value: AbiTraceType::None,
            sub_calls: None,
        });
        Ok(())
    })
}

/// sets the executable bytecode of the current address
pub(crate) fn assembly_script_set_bytecode(
    mut ctx: FunctionEnvMut<ASEnv>,
    bytecode: i32,
) -> ABIResult<()> {
    abi_with_memory!(ctx, assembly_script_set_bytecode, |memory| {
        let bytecode_raw = read_buffer(&memory, &ctx, bytecode)?;

        ctx.data().interface.raw_set_bytecode(&bytecode_raw)?;
        #[cfg(feature = "execution-trace")]
        ctx.data_mut().trace.push(AbiTrace {
            name: "assembly_script_set_bytecode".to_string(),
            params: vec![into_trace_value!(bytecode_raw)],
            return_value: AbiTraceType::None,
            sub_calls: None,
        });
        Ok(())
    })
}

/// get bytecode of the current address
pub(crate) fn assembly_script_get_bytecode(mut ctx: FunctionEnvMut<ASEnv>) -> ABIResult<i32> {
    abi!(ctx, assembly_script_get_bytecode, {
        let data = ctx.data().interface.raw_get_bytecode()?;
        let ffi_env = ctx.data().get_ffi_env().clone();
        let ptr = BufferPtr::alloc(&data, &ffi_env, &mut ctx)?.offset() as i32;
        #[cfg(feature = "execution-trace")]
        ctx.data_mut().trace.push(AbiTrace {
            name: "assembly_script_get_bytecode".to_string(),
            params: vec![],
            return_value: data.into(),
            sub_calls: None,
        });
        Ok(ptr)
    })
}

/// get bytecode of the target address
pub(crate) fn assembly_script_get_bytecode_for(
    mut ctx: FunctionEnvMut<ASEnv>,
    address: i32,
) -> ABIResult<i32> {
    abi_with_memory!(ctx, assembly_script_get_bytecode_for, |memory| {
        let address = read_string(&memory, &ctx, address)?;
        let data = ctx.data().interface.raw_get_bytecode_for(&address)?;
        let ffi_env = ctx.data().get_ffi_env().clone();
        let ptr = BufferPtr::alloc(&data, &ffi_env, &mut ctx)?.offset() as i32;
        #[cfg(feature = "execution-trace")]
        ctx.data_mut().trace.push(AbiTrace {
            name: "assembly_script_get_bytecode_for".to_string(),
            params: vec![into_trace_value!(address)],
            return_value: data.into(),
            sub_calls: None,
        });
        Ok(ptr)
    })
}

/// execute `function` of the given bytecode in the current context
pub(crate) fn assembly_script_local_execution(
    mut ctx: FunctionEnvMut<ASEnv>,
    bytecode: i32,
    function: i32,
    param: i32,
) -> ABIResult<i32> {
    abi_with_memory!(ctx, assembly_script_local_execution, |memory| {
        let bytecode = read_buffer(&memory, &ctx, bytecode)?;
        let function = read_string(&memory, &ctx, function)?;
        let param = read_buffer(&memory, &ctx, param)?;
        let response = local_call(&mut ctx, &bytecode, &function, &param, true)?;
        let ffi_env = ctx.data().get_ffi_env().clone();
        let res = match BufferPtr::alloc(&response.ret, &ffi_env, &mut ctx) {
            Ok(ret) => Ok(ret.offset() as i32),
            _ => abi_bail!(format!(
                "Cannot allocate response in local call of {}",
                function
            )),
        };
        #[cfg(feature = "execution-trace")]
        ctx.data_mut().trace.push(AbiTrace {
            name: "assembly_script_local_execution".to_string(),
            params: vec![
                into_trace_value!(bytecode),
                into_trace_value!(function),
                into_trace_value!(param),
            ],
            return_value: response.ret.clone().into(),
            sub_calls: Some(response.trace),
        });
        res
    })
}

/// execute `function` of the bytecode located at `address` in the current
/// context
pub(crate) fn assembly_script_local_call(
    mut ctx: FunctionEnvMut<ASEnv>,
    address: i32,
    function: i32,
    param: i32,
) -> ABIResult<i32> {
    abi_with_memory!(ctx, assembly_script_local_call, |memory| {
        let address = &read_string(&memory, &ctx, address)?;
        let bytecode = ctx.data().interface.raw_get_bytecode_for(address)?;
        let function = read_string(&memory, &ctx, function)?;
        let param = read_buffer(&memory, &ctx, param)?;

        let response = local_call(&mut ctx, &bytecode, &function, &param, false)?;
        let ffi_env = ctx.data().get_ffi_env().clone();
        let res = match BufferPtr::alloc(&response.ret, &ffi_env, &mut ctx) {
            Ok(ret) => Ok(ret.offset() as i32),
            _ => abi_bail!(format!(
                "Cannot allocate response in local call of {}",
                function
            )),
        };
        #[cfg(feature = "execution-trace")]
        ctx.data_mut().trace.push(AbiTrace {
            name: "assembly_script_local_call".to_string(),
            params: vec![
                into_trace_value!(bytecode),
                into_trace_value!(function),
                into_trace_value!(param),
            ],
            return_value: response.ret.clone().into(),
            sub_calls: Some(response.trace),
        });

        res
    })
}

/// Check whether or not the caller has write access in the current context
pub fn assembly_script_caller_has_write_access(mut ctx: FunctionEnvMut<ASEnv>) -> ABIResult<i32> {
    abi!(ctx, assembly_script_caller_has_write_access, {
        let has_write_access = ctx.data().interface.caller_has_write_access()?;
        #[cfg(feature = "execution-trace")]
        ctx.data_mut().trace.push(AbiTrace {
            name: "assembly_script_caller_has_write_access".to_string(),
            params: vec![],
            return_value: has_write_access.into(),
            sub_calls: None,
        });
        Ok(has_write_access as i32)
    })
}

/// Check whether the given function exists at the given address
pub fn assembly_script_function_exists(
    mut ctx: FunctionEnvMut<ASEnv>,
    address: i32,
    function: i32,
) -> ABIResult<i32> {
    abi_with_memory!(ctx, assembly_script_function_exists, |memory| {
        let address = read_string(&memory, &ctx, address)?;
        let function = read_string(&memory, &ctx, function)?;
        let function_exists = function_exists(&mut ctx, &address, &function)?;
        #[cfg(feature = "execution-trace")]
        ctx.data_mut().trace.push(AbiTrace {
            name: "assembly_script_function_exists".to_string(),
            params: vec![into_trace_value!(address), into_trace_value!(function)],
            return_value: function_exists.into(),
            sub_calls: None,
        });
        Ok(function_exists as i32)
    })
}

/// Return current chain id
pub(crate) fn assembly_script_chain_id(mut ctx: FunctionEnvMut<ASEnv>) -> ABIResult<u64> {
    abi!(ctx, assembly_script_chain_id, {
        let chain_id = ctx.data().interface.chain_id()?;
        #[cfg(feature = "execution-trace")]
        ctx.data_mut().trace.push(AbiTrace {
            name: "assembly_script_chain_id".to_string(),
            params: vec![],
            return_value: chain_id.into(),
            sub_calls: None,
        });
        Ok(chain_id)
    })
}

/// Return the price in nMAS to book an deferred call space in a specific slot.
pub(crate) fn assembly_script_get_deferred_call_quote(
    mut ctx: FunctionEnvMut<ASEnv>,
    deferred_call_period: i64,
    deferred_call_thread: i32,
    max_gas: i64,
    params_size: i64,
) -> ABIResult<u64> {
    let asc_slot: (u64, u8) = match (
        deferred_call_period.try_into(),
        deferred_call_thread.try_into(),
    ) {
        (Ok(p), Ok(t)) => (p, t),
        (Err(_), _) => abi_bail!("negative validity end period"),
        (_, Err(_)) => abi_bail!("invalid validity end thread"),
    };

    let max_gas: u64 = match max_gas.try_into() {
        Ok(g) => g,
        Err(_) => abi_bail!("negative max gas"),
    };

    let params_size: u64 = match params_size.try_into() {
        Ok(p) => p,
        Err(_) => abi_bail!("negative params size"),
    };

    abi!(ctx, assembly_script_get_deferred_call_quote, {
        let (available, mut price) =
            ctx.data()
                .interface
                .get_deferred_call_quote(asc_slot, max_gas, params_size)?;
        if !available {
            price = 0;
        }
        #[cfg(feature = "execution-trace")]
        ctx.data_mut().trace.push(AbiTrace {
            name: "assembly_script_get_deferred_call_quote".to_string(),
            params: vec![
                into_trace_value!(deferred_call_period),
                into_trace_value!(deferred_call_thread),
                into_trace_value!(max_gas),
            ],
            return_value: price.into(),
            sub_calls: None,
        });
        Ok(price)
    })
}

/// Register a new deferred call in the target slot with the given parameters.
#[allow(clippy::too_many_arguments)]
pub(crate) fn assembly_script_deferred_call_register(
    mut ctx: FunctionEnvMut<ASEnv>,
    target_address: i32,
    target_function: i32,
    target_period: i64,
    target_thread: i32,
    max_gas: i64,
    params: i32,
    raw_coins: i64,
) -> ABIResult<i32> {
    let asc_target_slot: (u64, u8) = match (target_period.try_into(), target_thread.try_into()) {
        (Ok(p), Ok(t)) => (p, t),
        (Err(_), _) => abi_bail!("negative validity end period"),
        (_, Err(_)) => abi_bail!("invalid validity end thread"),
    };

    let max_gas: u64 = match max_gas.try_into() {
        Ok(g) => g,
        Err(_) => abi_bail!("negative max gas"),
    };

    let raw_coins: u64 = match raw_coins.try_into() {
        Ok(c) => c,
        Err(_) => abi_bail!("negative coins"),
    };

    abi_with_memory!(ctx, assembly_script_deferred_call_register, |memory| {
        let target_address = read_string(&memory, &ctx, target_address)?;
        let target_function = read_string(&memory, &ctx, target_function)?;
        let params = read_buffer(&memory, &ctx, params)?;
        let response = ctx.data().interface.deferred_call_register(
            &target_address,
            &target_function,
            asc_target_slot,
            max_gas as u64,
            &params,
            raw_coins as u64,
        )?;
        let ffi_env = ctx.data().get_ffi_env().clone();
        let ptr = StringPtr::alloc(&response, &ffi_env, &mut ctx)?.offset() as i32;
        #[cfg(feature = "execution-trace")]
        ctx.data_mut().trace.push(AbiTrace {
            name: "assembly_script_deferred_call_register".to_string(),
            params: vec![
                into_trace_value!(target_address),
                into_trace_value!(target_function),
                into_trace_value!(target_period),
                into_trace_value!(target_thread),
                into_trace_value!(max_gas as u64),
                into_trace_value!(raw_coins as u64),
                into_trace_value!(params),
            ],
            return_value: response.to_owned().into(),
            sub_calls: None,
        });
        Ok(ptr)
    })
}

/// Check if an deferred call exists with the given deferred_call_id (exists meaning to be executed in the future).
pub(crate) fn assembly_script_deferred_call_exists(
    mut ctx: FunctionEnvMut<ASEnv>,
    deferred_id: i32,
) -> ABIResult<i32> {
    abi_with_memory!(ctx, assembly_script_deferred_call_exists, |memory| {
        let asc_id = read_string(&memory, &ctx, deferred_id)?;
        let exists = ctx.data().interface.deferred_call_exists(&asc_id)?;
        #[cfg(feature = "execution-trace")]
        ctx.data_mut().trace.push(AbiTrace {
            name: "assembly_script_deferred_call_exists".to_string(),
            params: vec![into_trace_value!(asc_id)],
            return_value: exists.into(),
            sub_calls: None,
        });
        Ok(exists as i32)
    })
}

/// Cancel an deferred call with the given deferred_call_id. This will reimburse the user with the coins they provided
pub(crate) fn assembly_script_deferred_call_cancel(
    mut ctx: FunctionEnvMut<ASEnv>,
    deferred_call_id: i32,
) -> ABIResult<()> {
    abi_with_memory!(ctx, assembly_script_deferred_call_cancel, |memory| {
        let deferred_id = read_string(&memory, &ctx, deferred_call_id)?;
        ctx.data().interface.deferred_call_cancel(&deferred_id)?;
        #[cfg(feature = "execution-trace")]
        ctx.data_mut().trace.push(AbiTrace {
            name: "assembly_script_deferred_call_cancel".to_string(),
            params: vec![into_trace_value!(deferred_id)],
            return_value: AbiTraceType::None,
            sub_calls: None,
        });
        Ok(())
    })
}

/// Assembly script builtin `abort` function.
///
/// It prints the origin filename, an error messag, the line and column.
#[allow(unused_macros)]
#[allow(unused_mut)]
pub fn assembly_script_abort(
    mut ctx: FunctionEnvMut<ASEnv>,
    message: StringPtr,
    filename: StringPtr,
    line: i32,
    col: i32,
) -> ABIResult<()> {
    let env = ctx.data();
    let memory = get_memory!(env);

    let message_ = message
        .read(memory, &ctx)
        .map_err(|e| wasmer::RuntimeError::new(e.to_string()));
    let filename_ = filename
        .read(memory, &ctx)
        .map_err(|e| wasmer::RuntimeError::new(e.to_string()));

    if message_.is_err() || filename_.is_err() {
        abi_bail!("aborting failed to load message or filename")
    }
    #[cfg(feature = "execution-trace")]
    ctx.data_mut().trace.push(AbiTrace {
        name: "assembly_script_abort".to_string(),
        params: vec![
            into_trace_value!(message_.clone().unwrap_or_default()),
            into_trace_value!(filename_.clone().unwrap_or_default()),
            into_trace_value!(line),
            into_trace_value!(col),
        ],
        return_value: AbiTraceType::None,
        sub_calls: None,
    });
    abi_bail!(format!(
        "error: {} at {}:{} col: {}",
        message_.unwrap(),
        filename_.unwrap(),
        line,
        col
    ));
}

/// Assembly script builtin `seed` function
pub fn assembly_script_seed(mut ctx: FunctionEnvMut<ASEnv>) -> ABIResult<f64> {
    if cfg!(feature = "gas_calibration") {
        let seed = match ctx.data().interface.unsafe_random_f64() {
            Ok(ret) => ret,
            _ => abi_bail!("failed to get random from interface"),
        };
        return Ok(seed);
    }

    abi!(ctx, assembly_script_seed, {
        let seed = match ctx.data().interface.unsafe_random_f64() {
            Ok(ret) => ret,
            _ => abi_bail!("failed to get random from interface"),
        };
        #[cfg(feature = "execution-trace")]
        ctx.data_mut().trace.push(AbiTrace {
            name: "assembly_script_seed".to_string(),
            params: vec![],
            return_value: seed.into(),
            sub_calls: None,
        });
        Ok(seed)
    })
}

/// Assembly script builtin `Date.now()`
pub fn assembly_script_date_now(mut ctx: FunctionEnvMut<ASEnv>) -> ABIResult<f64> {
    if cfg!(feature = "gas_calibration") {
        let utime = match ctx.data().interface.get_time() {
            Ok(time) => time,
            _ => abi_bail!("failed to get time from interface"),
        };
        return Ok(utime as f64);
    }

    abi!(ctx, assembly_script_date_now, {
        let utime = match ctx.data().interface.get_time() {
            Ok(time) => time,
            _ => abi_bail!("failed to get time from interface"),
        };
        let ret = utime as f64;
        #[cfg(feature = "execution-trace")]
        ctx.data_mut().trace.push(AbiTrace {
            name: "assembly_script_date_now".to_string(),
            params: vec![],
            return_value: ret.into(),
            sub_calls: None,
        });
        Ok(ret)
    })
}

/// Assembly script builtin `console.log()`.
pub fn assembly_script_console_log(
    mut ctx: FunctionEnvMut<ASEnv>,
    message: StringPtr,
) -> ABIResult<()> {
    if cfg!(feature = "gas_calibration") {
        let memory = match ctx.data().get_ffi_env().memory.as_ref() {
            Some(m) => m.clone(),
            None => abi_bail!("No memory"),
        };
        return assembly_script_console(&memory, &mut ctx, message, "LOG");
    }
    abi_with_memory!(ctx, assembly_script_console_log, |memory| {
        assembly_script_console(&memory, &mut ctx, message, "LOG")
    })
}

/// Assembly script builtin `console.info()`.
pub fn assembly_script_console_info(
    mut ctx: FunctionEnvMut<ASEnv>,
    message: StringPtr,
) -> ABIResult<()> {
    if cfg!(feature = "gas_calibration") {
        let memory = match ctx.data().get_ffi_env().memory.as_ref() {
            Some(m) => m.clone(),
            None => abi_bail!("No memory"),
        };
        return assembly_script_console(&memory, &mut ctx, message, "INFO");
    }
    abi_with_memory!(ctx, assembly_script_console_info, |memory| {
        assembly_script_console(&memory, &mut ctx, message, "INFO")
    })
}

/// Assembly script builtin `console.warn()`.
pub fn assembly_script_console_warn(
    mut ctx: FunctionEnvMut<ASEnv>,
    message: StringPtr,
) -> ABIResult<()> {
    if cfg!(feature = "gas_calibration") {
        let memory = match ctx.data().get_ffi_env().memory.as_ref() {
            Some(m) => m.clone(),
            None => abi_bail!("No memory"),
        };
        return assembly_script_console(&memory, &mut ctx, message, "WARN");
    }
    abi_with_memory!(ctx, assembly_script_console_warn, |memory| {
        assembly_script_console(&memory, &mut ctx, message, "WARN")
    })
}

/// Assembly script builtin `console.debug()`.
pub fn assembly_script_console_debug(
    mut ctx: FunctionEnvMut<ASEnv>,
    message: StringPtr,
) -> ABIResult<()> {
    if cfg!(feature = "gas_calibration") {
        let memory = match ctx.data().get_ffi_env().memory.as_ref() {
            Some(m) => m.clone(),
            None => abi_bail!("No memory"),
        };
        return assembly_script_console(&memory, &mut ctx, message, "DEBUG");
    }
    abi_with_memory!(ctx, assembly_script_console_debug, |memory| {
        assembly_script_console(&memory, &mut ctx, message, "DEBUG")
    })
}

/// Assembly script builtin `console.error()`.
pub fn assembly_script_console_error(
    mut ctx: FunctionEnvMut<ASEnv>,
    message: StringPtr,
) -> ABIResult<()> {
    if cfg!(feature = "gas_calibration") {
        let memory = match ctx.data().get_ffi_env().memory.as_ref() {
            Some(m) => m.clone(),
            None => abi_bail!("No memory"),
        };
        return assembly_script_console(&memory, &mut ctx, message, "ERROR");
    }
    abi_with_memory!(ctx, assembly_script_console_error, |memory| {
        assembly_script_console(&memory, &mut ctx, message, "ERROR")
    })
}

/// Assembly script console functions
#[allow(unused_macros)]
#[allow(unused_mut)]
fn assembly_script_console(
    memory: &Memory,
    ctx: &mut FunctionEnvMut<ASEnv>,
    message: StringPtr,
    prefix: &str,
) -> ABIResult<()> {
    // Helper function - no gas accounting, memory passed in
    let message = prefix
        .to_string()
        .add(" | ")
        .add(&message.read(memory, ctx)?);

    ctx.data().interface.generate_event(message.clone())?;

    #[cfg(feature = "execution-trace")]
    ctx.data_mut().trace.push(AbiTrace {
        name: "assembly_script_console_error".to_string(),
        params: vec![into_trace_value!(message)],
        return_value: AbiTraceType::None,
        sub_calls: None,
    });
    Ok(())
}

/// Assembly script builtin `trace()`.
#[allow(clippy::too_many_arguments)]
pub fn assembly_script_trace(
    mut ctx: FunctionEnvMut<ASEnv>,
    message: StringPtr,
    n: i32,
    a0: f64,
    a1: f64,
    a2: f64,
    a3: f64,
    a4: f64,
) -> ABIResult<()> {
    if cfg!(feature = "gas_calibration") {
        let memory = ctx
            .data()
            .get_ffi_env()
            .memory
            .as_ref()
            .expect("Failed to get memory on env")
            .clone();
        let message_str = message.read(&memory, &ctx)?;
        let message_for_event = match n {
            1 => format!("msg: {}, a0: {}", message_str, a0),
            2 => format!("msg: {}, a0: {}, a1: {}", message_str, a0, a1),
            3 => format!("msg: {}, a0: {}, a1: {}, a2: {}", message_str, a0, a1, a2),
            4 => format!(
                "msg: {}, a0: {}, a1: {}, a2: {}, a3: {}",
                message_str, a0, a1, a2, a3
            ),
            5 => format!(
                "msg: {}, a0: {}, a1: {}, a2: {}, a3: {}, a4: {}",
                message_str, a0, a1, a2, a3, a4
            ),
            _ => message_str,
        };
        ctx.data().interface.generate_event(message_for_event)?;
        return Ok(());
    }

    abi_with_memory!(ctx, assembly_script_trace, |memory| {
        let message_str = message.read(&memory, &ctx)?;
        let message_for_event = match n {
            1 => format!("msg: {}, a0: {}", message_str, a0),
            2 => format!("msg: {}, a0: {}, a1: {}", message_str, a0, a1),
            3 => format!("msg: {}, a0: {}, a1: {}, a2: {}", message_str, a0, a1, a2),
            4 => format!(
                "msg: {}, a0: {}, a1: {}, a2: {}, a3: {}",
                message_str, a0, a1, a2, a3
            ),
            5 => format!(
                "msg: {}, a0: {}, a1: {}, a2: {}, a3: {}, a4: {}",
                message_str, a0, a1, a2, a3, a4
            ),
            _ => message_str,
        };
        ctx.data()
            .interface
            .generate_event(message_for_event.clone())?;
        #[cfg(feature = "execution-trace")]
        ctx.data_mut().trace.push(AbiTrace {
            name: "assembly_script_trace".to_string(),
            params: vec![into_trace_value!(message_for_event)],
            return_value: AbiTraceType::None,
            sub_calls: None,
        });
        Ok(())
    })
}

/// Assembly script builtin `process.exit()`.
pub fn assembly_script_process_exit(_ctx: FunctionEnvMut<ASEnv>, exit_code: i32) -> ABIResult<()> {
    abi_bail!(format!("exit with code: {}", exit_code));
}

/// Tooling that reads a buffer (Vec<u8>) in memory
fn read_buffer(memory: &Memory, store: &impl AsStoreRef, offset: i32) -> ABIResult<Vec<u8>> {
    Ok(BufferPtr::new(offset as u32).read(memory, store)?)
}

/// Tooling, return a string from a given offset
fn read_string(memory: &Memory, store: &impl AsStoreRef, ptr: i32) -> ABIResult<String> {
    Ok(StringPtr::new(ptr as u32).read(memory, store)?)
}

/// Tooling, return a pointer offset of a serialized list in json
fn alloc_string_array(ctx: &mut FunctionEnvMut<ASEnv>, vec: &[String]) -> ABIResult<i32> {
    let addresses = serde_json::to_string(vec).map_err(|e| ABIError::SerdeError(e.to_string()))?;
    let ffi_env = ctx.data().get_ffi_env().clone();
    Ok(StringPtr::alloc(&addresses, &ffi_env, ctx)?.offset() as i32)
}

/// Flatten a Vec<Vec<u8>> (or anything that can be turned into an iterator) to
/// a Vec<u8> with the format: L (32 bits LE) V1_L (8 bits) V1 (8bits * V1_L),
/// V2_L ... VN (8 bits * VN_L)
/// Edge cases:
/// Serializing an empty vec![] will return an empty vec![] and not [0, 0, 0, 0]
/// See also unit tests: test_ser_edge_cases
fn ser_bytearray_vec<'a, I>(data: I, data_len: usize, max_length: usize) -> ABIResult<Vec<u8>>
where
    I: IntoIterator<Item = &'a Vec<u8>>,
{
    if data_len == 0 {
        return Ok(Vec::new());
    }

    if data_len > max_length {
        abi_bail!("Too many entries in the datastore");
    }

    // pre alloc with max capacity
    let mut buffer = Vec::with_capacity(4 + (data_len * (1 + 255)));

    let entry_count = u32::try_from(data_len).unwrap();
    buffer.extend_from_slice(&entry_count.to_le_bytes());

    for key in data.into_iter() {
        let k_len = match u8::try_from(key.len()) {
            Ok(l) => l,
            Err(_) => abi_bail!("Some Datastore keys are too long"),
        };
        buffer.push(k_len);
        buffer.extend_from_slice(&key[..]);
    }

    Ok(buffer)
}

/// performs a sha256 hash on byte array and returns the hash as byte array
pub(crate) fn assembly_script_hash_sha256(
    mut ctx: FunctionEnvMut<ASEnv>,
    bytes: i32,
) -> ABIResult<i32> {
    abi_with_memory!(ctx, assembly_script_hash_sha256, |memory| {
        let bytes = read_buffer(&memory, &ctx, bytes)?;
        let hash = ctx.data().interface.hash_sha256(&bytes)?.to_vec();
        let ffi_env = ctx.data().get_ffi_env().clone();
        let ptr = BufferPtr::alloc(&hash, &ffi_env, &mut ctx)?.offset();
        Ok(ptr as i32)
    })
}

#[cfg(test)]
mod tests {
    use crate::as_execution::abi::ser_bytearray_vec;

    #[test]
    fn test_ser() {
        let vb: Vec<Vec<u8>> = vec![vec![1, 2, 3], vec![255]];

        let vb_ser = ser_bytearray_vec(&vb, vb.len(), 10).unwrap();
        // Expected:
        // L: 2, 0, 0, 0 == 2 as u32 (little endian)
        // V1L: 3 as u8
        // V1 values: 1, 2, 3
        // V2L: 1 as u8
        // V2 values: 255
        assert_eq!(vb_ser, [2, 0, 0, 0, 3, 1, 2, 3, 1, 255]);
    }

    #[test]
    fn test_ser_edge_cases() {
        // Serializing some values with one as an empty vec
        let vb: Vec<Vec<u8>> = vec![vec![1, 2, 3], vec![]];

        let vb_ser = ser_bytearray_vec(&vb, vb.len(), 10).unwrap();
        // Expected:
        // L: 2, 0, 0, 0 == 2 as u32 (little endian)
        // V1L: 3 as u8
        // V1 values: 1, 2, 3 (little endian)
        // V2L: 0 as u8
        // V2 values: None
        assert_eq!(vb_ser, [2, 0, 0, 0, 3, 1, 2, 3, 0]);

        // Serializing with invalid max_length
        let vb_ser = ser_bytearray_vec(&vb, vb.len(), 1);
        assert!(vb_ser.is_err());

        // Serializing an empty vec
        let vb: Vec<Vec<u8>> = vec![];
        let vb_ser = ser_bytearray_vec(&vb, vb.len(), 10).unwrap();
        let empty_vec: Vec<u8> = vec![];
        assert_eq!(vb_ser, empty_vec);

        // A huge vec to serialize
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
