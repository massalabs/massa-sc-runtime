//! *abi_impl.rs* contains all the implementation (and some tools as
//! abi_bail!) of the massa abi.
//!
//! The ABIs are the imported function / object declared in the webassembly
//! module. You can look at the other side of the mirror in `massa.ts` and the
//! rust side in `execution_impl.rs`.
use crate::env::{
    get_memory, get_remaining_points, sub_remaining_gas, sub_remaining_gas_with_mult, ASEnv,
    MassaEnv,
};
use crate::middlewares::gas_calibration::param_size_update;
use crate::settings;
use as_ffi_bindings::{BufferPtr, Read as ASRead, StringPtr, Write as ASWrite};
use function_name::named;
use wasmer::Memory;

use super::common::{abi_bail, call_module, create_sc, ABIResult};
use super::{create_instance, get_module, local_call, MassaModule};

/// Get the coins that have been made available for a specific purpose for the current call.
pub(crate) fn assembly_script_get_call_coins(env: &ASEnv) -> ABIResult<i64> {
    sub_remaining_gas(env, settings::metering_get_call_coins())?;
    Ok(env.get_interface().get_call_coins()? as i64)
}

/// Transfer an amount from the address on the current call stack to a target address.
#[named]
pub(crate) fn assembly_script_transfer_coins(
    env: &ASEnv,
    to_address: i32,
    raw_amount: i64,
) -> ABIResult<()> {
    sub_remaining_gas(env, settings::metering_transfer())?;
    if raw_amount.is_negative() {
        abi_bail!("Negative raw amount.");
    }
    let memory = get_memory!(env);
    let to_address = &get_string(memory, to_address)?;
    if cfg!(feature = "gas_calibration") {
        let fname = format!("massa.{}:0", function_name!());
        param_size_update(env, &fname, to_address.len(), true);
    }
    Ok(env
        .get_interface()
        .transfer_coins(to_address, raw_amount as u64)?)
}

/// Transfer an amount from the specified address to a target address.
#[named]
pub(crate) fn assembly_script_transfer_coins_for(
    env: &ASEnv,
    from_address: i32,
    to_address: i32,
    raw_amount: i64,
) -> ABIResult<()> {
    sub_remaining_gas(env, settings::metering_transfer())?;
    if raw_amount.is_negative() {
        abi_bail!("Negative raw amount.");
    }
    let memory = get_memory!(env);
    let from_address = &get_string(memory, from_address)?;
    let to_address = &get_string(memory, to_address)?;
    if cfg!(feature = "gas_calibration") {
        let fname = format!("massa.{}:0", function_name!());
        param_size_update(env, &fname, from_address.len(), true);
        let fname = format!("massa.{}:1", function_name!());
        param_size_update(env, &fname, to_address.len(), true);
    }
    Ok(env
        .get_interface()
        .transfer_coins_for(from_address, to_address, raw_amount as u64)?)
}

pub(crate) fn assembly_script_get_balance(env: &ASEnv) -> ABIResult<i64> {
    sub_remaining_gas(env, settings::metering_get_balance())?;
    Ok(env.get_interface().get_balance()? as i64)
}

#[named]
pub(crate) fn assembly_script_get_balance_for(env: &ASEnv, address: i32) -> ABIResult<i64> {
    sub_remaining_gas(env, settings::metering_get_balance())?;
    let memory = get_memory!(env);
    let address = &get_string(memory, address)?;
    if cfg!(feature = "gas_calibration") {
        let fname = format!("massa.{}:0", function_name!());
        param_size_update(env, &fname, address.len(), true);
    }
    Ok(env.get_interface().get_balance_for(address)? as i64)
}

/// Raw call that have the right type signature to be able to be call a module
/// directly form AssemblyScript:
#[named]
pub(crate) fn assembly_script_call(
    env: &ASEnv,
    address: i32,
    function: i32,
    param: i32,
    call_coins: i64,
) -> ABIResult<i32> {
    sub_remaining_gas(env, settings::metering_call())?;
    let memory = get_memory!(env);
    let address = &get_string(memory, address)?;
    let function = &get_string(memory, function)?;
    let param = &read_buffer(memory, param)?;

    if cfg!(feature = "gas_calibration") {
        let fname = format!("massa.{}:0", function_name!());
        param_size_update(env, &fname, address.len(), true);
        let fname = format!("massa.{}:1", function_name!());
        param_size_update(env, &fname, function.len(), true);
        let fname = format!("massa.{}:2", function_name!());
        param_size_update(env, &fname, param.len(), true);
    }

    let response = call_module(env, address, function, param, call_coins)?;
    match BufferPtr::alloc(&response.ret, env.get_wasm_env()) {
        Ok(ret) => Ok(ret.offset() as i32),
        _ => abi_bail!(format!(
            "Cannot allocate response in call {}::{}",
            address, function
        )),
    }
}

pub(crate) fn assembly_script_get_remaining_gas(env: &ASEnv) -> ABIResult<i64> {
    sub_remaining_gas(env, settings::metering_remaining_gas())?;
    Ok(get_remaining_points(env)? as i64)
}

/// Create an instance of VM from a module with a
/// given interface, an operation number limit and a webassembly module
///
/// An utility print function to write on stdout directly from AssemblyScript:
#[named]
pub(crate) fn assembly_script_print(env: &ASEnv, arg: i32) -> ABIResult<()> {
    sub_remaining_gas(env, settings::metering_print())?;
    let memory = get_memory!(env);
    let message = get_string(memory, arg)?;

    if cfg!(feature = "gas_calibration") {
        let fname = format!("massa.{}:0", function_name!());
        param_size_update(env, &fname, message.len(), true);
    }

    env.get_interface().print(&message)?;
    Ok(())
}

/// Get the operation datastore keys (aka entries)
pub(crate) fn assembly_script_get_op_keys(env: &ASEnv) -> ABIResult<i32> {
    match env.get_interface().get_op_keys() {
        Err(err) => abi_bail!(err),
        Ok(keys) => {
            sub_remaining_gas_with_mult(
                env,
                keys.iter().fold(0, |acc, v_| acc + v_.len()),
                settings::get_op_keys_mult(),
            )?;
            let fmt_keys =
                ser_bytearray_vec(&keys, keys.len(), settings::max_op_datastore_entry_count())?;
            let ptr = pointer_from_bytearray(env, &fmt_keys)?.offset();
            Ok(ptr as i32)
        }
    }
}

/// Check if a key is present in operation datastore
#[named]
pub(crate) fn assembly_script_has_op_key(env: &ASEnv, key: i32) -> ABIResult<i32> {
    let memory = get_memory!(env);
    let key_bytes = read_buffer_and_sub_gas(env, memory, key, settings::has_op_key_mult())?;
    if cfg!(feature = "gas_calibration") {
        let fname = format!("massa.{}:0", function_name!());
        param_size_update(env, &fname, key_bytes.len(), true);
    }

    Ok(env.get_interface().has_op_key(&key_bytes)? as i32)
}

/// Get the operation datastore value associated to given key
#[named]
pub(crate) fn assembly_script_get_op_data(env: &ASEnv, key: i32) -> ABIResult<i32> {
    let memory = get_memory!(env);
    let key_bytes = read_buffer_and_sub_gas(env, memory, key, settings::get_op_data_mult())?;
    if cfg!(feature = "gas_calibration") {
        let fname = format!("massa.{}:0", function_name!());
        param_size_update(env, &fname, key_bytes.len(), true);
    }
    let data = env.get_interface().get_op_data(&key_bytes)?;
    let ptr = pointer_from_bytearray(env, &data)?.offset() as i32;
    Ok(ptr)
}

/// Read a bytecode string, representing the webassembly module binary encoded
/// with in base64.
#[named]
pub(crate) fn assembly_script_create_sc(env: &ASEnv, bytecode: i32) -> ABIResult<i32> {
    let memory = get_memory!(env);
    let bytecode: Vec<u8> =
        read_buffer_and_sub_gas(env, memory, bytecode, settings::metering_create_sc_mult())?;
    if cfg!(feature = "gas_calibration") {
        let fname = format!("massa.{}:0", function_name!());
        param_size_update(env, &fname, bytecode.len(), true);
    }
    let address = create_sc(env, &bytecode)?;
    Ok(StringPtr::alloc(&address, env.get_wasm_env())?.offset() as i32)
}

/// performs a hash on a string and returns the bs58check encoded hash
#[named]
pub(crate) fn assembly_script_hash(env: &ASEnv, value: i32) -> ABIResult<i32> {
    sub_remaining_gas(env, settings::metering_hash_const())?;
    let memory = get_memory!(env);
    let value = read_string_and_sub_gas(env, memory, value, settings::metering_hash_per_byte())?;
    if cfg!(feature = "gas_calibration") {
        let fname = format!("massa.{}:0", function_name!());
        param_size_update(env, &fname, value.len(), true);
    }
    let hash = env.get_interface().hash(value.as_bytes())?;
    Ok(pointer_from_string(env, &hash)?.offset() as i32)
}

/// Get keys (aka entries) in the datastore
pub(crate) fn assembly_script_get_keys(env: &ASEnv) -> ABIResult<i32> {
    let keys = env.get_interface().get_keys()?;
    sub_remaining_gas_with_mult(
        env,
        keys.iter().fold(0, |acc, v_| acc + v_.len()),
        settings::get_keys_mult(),
    )?;
    let fmt_keys = ser_bytearray_vec(&keys, keys.len(), settings::max_datastore_entry_count())?;
    let ptr = pointer_from_bytearray(env, &fmt_keys)?.offset();
    Ok(ptr as i32)
}

/// Get keys (aka entries) in the datastore
pub(crate) fn assembly_script_get_keys_for(env: &ASEnv, address: i32) -> ABIResult<i32> {
    let memory = get_memory!(env);
    let address = get_string(memory, address)?;
    let keys = env.get_interface().get_keys_for(&address)?;
    sub_remaining_gas_with_mult(
        env,
        keys.iter().fold(0, |acc, v_| acc + v_.len()),
        settings::get_keys_mult(),
    )?;
    let fmt_keys = ser_bytearray_vec(&keys, keys.len(), settings::max_datastore_entry_count())?;
    let ptr = pointer_from_bytearray(env, &fmt_keys)?.offset();
    Ok(ptr as i32)
}

/// sets a key-indexed data entry in the datastore, overwriting existing values if any
pub(crate) fn assembly_script_set_data(env: &ASEnv, key: i32, value: i32) -> ABIResult<()> {
    sub_remaining_gas(env, settings::metering_set_data_const())?;
    let memory = get_memory!(env);
    let key = read_buffer_and_sub_gas(env, memory, key, settings::metering_set_data_key_mult())?;
    let value =
        read_buffer_and_sub_gas(env, memory, value, settings::metering_set_data_value_mult())?;

    if cfg!(feature = "gas_calibration") {
        param_size_update(env, "massa.assembly_script_set_data:0", key.len(), false);
        param_size_update(env, "massa.assembly_script_set_data:1", value.len(), false);
    }

    env.get_interface().raw_set_data(&key, &value)?;
    Ok(())
}

/// appends data to a key-indexed data entry in the datastore, fails if the entry does not exist
#[named]
pub(crate) fn assembly_script_append_data(env: &ASEnv, key: i32, value: i32) -> ABIResult<()> {
    sub_remaining_gas(env, settings::metering_append_data_const())?;
    let memory = get_memory!(env);
    let key = read_buffer_and_sub_gas(env, memory, key, settings::metering_append_data_key_mult())?;
    let value = read_buffer_and_sub_gas(
        env,
        memory,
        value,
        settings::metering_append_data_value_mult(),
    )?;
    if cfg!(feature = "gas_calibration") {
        let fname = format!("massa.{}:0", function_name!());
        param_size_update(env, &fname, key.len(), true);
        let fname = format!("massa.{}:1", function_name!());
        param_size_update(env, &fname, value.len(), true);
    }
    env.get_interface().raw_append_data(&key, &value)?;
    Ok(())
}

/// gets a key-indexed data entry in the datastore, failing if non-existent
#[named]
pub(crate) fn assembly_script_get_data(env: &ASEnv, key: i32) -> ABIResult<i32> {
    sub_remaining_gas(env, settings::metering_get_data_const())?;
    let memory = get_memory!(env);
    let key = read_buffer_and_sub_gas(env, memory, key, settings::metering_get_data_key_mult())?;
    if cfg!(feature = "gas_calibration") {
        let fname = format!("massa.{}:0", function_name!());
        param_size_update(env, &fname, key.len(), true);
    }
    let data = env.get_interface().raw_get_data(&key)?;
    sub_remaining_gas_with_mult(env, data.len(), settings::metering_get_data_value_mult())?;
    Ok(pointer_from_bytearray(env, &data)?.offset() as i32)
}

/// checks if a key-indexed data entry exists in the datastore
#[named]
pub(crate) fn assembly_script_has_data(env: &ASEnv, key: i32) -> ABIResult<i32> {
    sub_remaining_gas(env, settings::metering_has_data_const())?;
    let memory = get_memory!(env);
    let key = read_buffer_and_sub_gas(env, memory, key, settings::metering_has_data_key_mult())?;
    if cfg!(feature = "gas_calibration") {
        let fname = format!("massa.{}:0", function_name!());
        param_size_update(env, &fname, key.len(), true);
    }
    Ok(env.get_interface().has_data(&key)? as i32)
}

/// deletes a key-indexed data entry in the datastore of the current address, fails if the entry is absent
#[named]
pub(crate) fn assembly_script_delete_data(env: &ASEnv, key: i32) -> ABIResult<()> {
    sub_remaining_gas(env, settings::metering_delete_data_const())?;
    let memory = get_memory!(env);
    let key = read_buffer_and_sub_gas(env, memory, key, settings::metering_delete_data_key_mult())?;
    if cfg!(feature = "gas_calibration") {
        let fname = format!("massa.{}:0", function_name!());
        param_size_update(env, &fname, key.len(), true);
    }
    env.get_interface().raw_delete_data(&key)?;
    Ok(())
}

/// Sets the value of a datastore entry of an arbitrary address, creating the entry if it does not exist.
/// Fails if the address does not exist.
#[named]
pub(crate) fn assembly_script_set_data_for(
    env: &ASEnv,
    address: i32,
    key: i32,
    value: i32,
) -> ABIResult<()> {
    sub_remaining_gas(env, settings::metering_set_data_const())?;
    let memory = get_memory!(env);
    let key = read_buffer_and_sub_gas(env, memory, key, settings::metering_set_data_key_mult())?;
    let value =
        read_buffer_and_sub_gas(env, memory, value, settings::metering_set_data_value_mult())?;
    let address = get_string(memory, address)?;
    if cfg!(feature = "gas_calibration") {
        let fname = format!("massa.{}:0", function_name!());
        param_size_update(env, &fname, address.len(), true);
        let fname = format!("massa.{}:1", function_name!());
        param_size_update(env, &fname, key.len(), true);
        let fname = format!("massa.{}:2", function_name!());
        param_size_update(env, &fname, value.len(), true);
    }
    env.get_interface()
        .raw_set_data_for(&address, &key, &value)?;
    Ok(())
}

/// Appends data to the value of a datastore entry of an arbitrary address, fails if the entry or address does not exist.
#[named]
pub(crate) fn assembly_script_append_data_for(
    env: &ASEnv,
    address: i32,
    key: i32,
    value: i32,
) -> ABIResult<()> {
    sub_remaining_gas(env, settings::metering_append_data_const())?;
    let memory = get_memory!(env);
    let key = read_buffer_and_sub_gas(env, memory, key, settings::metering_append_data_key_mult())?;
    let value = read_buffer_and_sub_gas(
        env,
        memory,
        value,
        settings::metering_append_data_value_mult(),
    )?;
    let address = get_string(memory, address)?;
    if cfg!(feature = "gas_calibration") {
        let fname = format!("massa.{}:0", function_name!());
        param_size_update(env, &fname, address.len(), true);
        let fname = format!("massa.{}:1", function_name!());
        param_size_update(env, &fname, key.len(), true);
        let fname = format!("massa.{}:2", function_name!());
        param_size_update(env, &fname, value.len(), true);
    }
    env.get_interface()
        .raw_append_data_for(&address, &key, &value)?;
    Ok(())
}

/// Gets the value of a datastore entry for an arbitrary address, fails if the entry or address does not exist
#[named]
pub(crate) fn assembly_script_get_data_for(env: &ASEnv, address: i32, key: i32) -> ABIResult<i32> {
    sub_remaining_gas(env, settings::metering_get_data_const())?;
    let memory = get_memory!(env);
    let address = get_string(memory, address)?;
    let key = read_buffer_and_sub_gas(env, memory, key, settings::metering_get_data_key_mult())?;
    if cfg!(feature = "gas_calibration") {
        let fname = format!("massa.{}:0", function_name!());
        param_size_update(env, &fname, address.len(), true);
        let fname = format!("massa.{}:1", function_name!());
        param_size_update(env, &fname, key.len(), true);
    }

    let data = env.get_interface().raw_get_data_for(&address, &key)?;
    sub_remaining_gas_with_mult(env, data.len(), settings::metering_get_data_value_mult())?;
    Ok(pointer_from_bytearray(env, &data)?.offset() as i32)
}

/// Deletes a datastore entry for an address. Fails if the entry or address does not exist.
#[named]
pub(crate) fn assembly_script_delete_data_for(
    env: &ASEnv,
    address: i32,
    key: i32,
) -> ABIResult<()> {
    sub_remaining_gas(env, settings::metering_delete_data_const())?;
    let memory = get_memory!(env);
    let address = get_string(memory, address)?;
    let key = read_buffer_and_sub_gas(env, memory, key, settings::metering_delete_data_key_mult())?;
    if cfg!(feature = "gas_calibration") {
        let fname = format!("massa.{}:0", function_name!());
        param_size_update(env, &fname, address.len(), true);
        let fname = format!("massa.{}:1", function_name!());
        param_size_update(env, &fname, key.len(), true);
    }
    env.get_interface().raw_delete_data_for(&address, &key)?;
    Ok(())
}

#[named]
pub(crate) fn assembly_script_has_data_for(env: &ASEnv, address: i32, key: i32) -> ABIResult<i32> {
    sub_remaining_gas(env, settings::metering_has_data_const())?;
    let memory = get_memory!(env);
    let address = get_string(memory, address)?;
    let key = read_buffer_and_sub_gas(env, memory, key, settings::metering_has_data_key_mult())?;
    if cfg!(feature = "gas_calibration") {
        let fname = format!("massa.{}:0", function_name!());
        param_size_update(env, &fname, address.len(), true);
        let fname = format!("massa.{}:1", function_name!());
        param_size_update(env, &fname, key.len(), true);
    }
    Ok(env.get_interface().has_data_for(&address, &key)? as i32)
}

pub(crate) fn assembly_script_get_owned_addresses_raw(env: &ASEnv) -> ABIResult<i32> {
    sub_remaining_gas(env, settings::metering_get_owned_addrs())?;
    let data = env.get_interface().get_owned_addresses()?;
    Ok(StringPtr::alloc(&data.join(";"), env.get_wasm_env())?.offset() as i32)
}

pub(crate) fn assembly_script_get_call_stack_raw(env: &ASEnv) -> ABIResult<i32> {
    sub_remaining_gas(env, settings::metering_get_call_stack())?;
    let data = env.get_interface().get_call_stack()?;
    Ok(StringPtr::alloc(&data.join(";"), env.get_wasm_env())?.offset() as i32)
}

pub(crate) fn assembly_script_get_owned_addresses(env: &ASEnv) -> ABIResult<i32> {
    sub_remaining_gas(env, settings::metering_get_owned_addrs())?;
    let data = env.get_interface().get_owned_addresses()?;
    alloc_string_array(env, &data)
}

pub(crate) fn assembly_script_get_call_stack(env: &ASEnv) -> ABIResult<i32> {
    sub_remaining_gas(env, settings::metering_get_call_stack())?;
    let data = env.get_interface().get_call_stack()?;
    alloc_string_array(env, &data)
}

#[named]
pub(crate) fn assembly_script_generate_event(env: &ASEnv, event: i32) -> ABIResult<()> {
    sub_remaining_gas(env, settings::metering_generate_event())?;
    let memory = get_memory!(env);
    let event = get_string(memory, event)?;
    if cfg!(feature = "gas_calibration") {
        let fname = format!("massa.{}:0", function_name!());
        param_size_update(env, &fname, event.len(), true);
    }
    env.get_interface().generate_event(event)?;
    Ok(())
}

/// verify a signature of data given a public key. Returns Ok(1) if correctly verified, otherwise Ok(0)
#[named]
pub(crate) fn assembly_script_signature_verify(
    env: &ASEnv,
    data: i32,
    signature: i32,
    public_key: i32,
) -> ABIResult<i32> {
    sub_remaining_gas(env, settings::metering_signature_verify_const())?;
    let memory = get_memory!(env);
    let data = read_string_and_sub_gas(
        env,
        memory,
        data,
        settings::metering_signature_verify_data_mult(),
    )?;
    let signature = get_string(memory, signature)?;
    let public_key = get_string(memory, public_key)?;
    if cfg!(feature = "gas_calibration") {
        let fname = format!("massa.{}:0", function_name!());
        param_size_update(env, &fname, data.len(), true);
        let fname = format!("massa.{}:1", function_name!());
        param_size_update(env, &fname, signature.len(), true);
        let fname = format!("massa.{}:2", function_name!());
        param_size_update(env, &fname, public_key.len(), true);
    }
    Ok(env
        .get_interface()
        .signature_verify(data.as_bytes(), &signature, &public_key)? as i32)
}

/// converts a public key to an address
#[named]
pub(crate) fn assembly_script_address_from_public_key(
    env: &ASEnv,
    public_key: i32,
) -> ABIResult<i32> {
    sub_remaining_gas(env, settings::metering_address_from_public_key())?;
    let memory = get_memory!(env);
    let public_key = get_string(memory, public_key)?;
    if cfg!(feature = "gas_calibration") {
        let fname = format!("massa.{}:0", function_name!());
        param_size_update(env, &fname, public_key.len(), true);
    }
    let addr = env.get_interface().address_from_public_key(&public_key)?;
    Ok(pointer_from_string(env, &addr)?.offset() as i32)
}

/// generates an unsafe random number
pub(crate) fn assembly_script_unsafe_random(env: &ASEnv) -> ABIResult<i64> {
    sub_remaining_gas(env, settings::metering_unsafe_random())?;
    Ok(env.get_interface().unsafe_random()?)
}

/// gets the current unix timestamp in milliseconds
pub(crate) fn assembly_script_get_time(env: &ASEnv) -> ABIResult<i64> {
    sub_remaining_gas(env, settings::metering_get_time())?;
    Ok(env.get_interface().get_time()? as i64)
}

/// sends an async message
#[allow(clippy::too_many_arguments)]
#[named]
pub(crate) fn assembly_script_send_message(
    env: &ASEnv,
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
    sub_remaining_gas(env, settings::metering_send_message())?;
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
    let memory = get_memory!(env);
    let target_address = &get_string(memory, target_address)?;
    let target_handler = &get_string(memory, target_handler)?;
    let data = &read_buffer(memory, data)?;
    if cfg!(feature = "gas_calibration") {
        let fname = format!("massa.{}:0", function_name!());
        param_size_update(env, &fname, target_address.len(), true);
        let fname = format!("massa.{}:1", function_name!());
        param_size_update(env, &fname, target_handler.len(), true);
        let fname = format!("massa.{}:2", function_name!());
        param_size_update(env, &fname, data.len(), true);
    }
    let filter_address_string = &get_string(memory, filter_address)?;
    let key = read_buffer_and_sub_gas(
        env,
        memory,
        filter_datastore_key,
        settings::metering_has_data_key_mult(),
    )?;
    let filter = match (filter_address_string.as_str(), key.as_slice()) {
        ("", _) => None,
        (addr, &[]) => Some((addr, None)),
        (addr, key) => Some((addr, Some(key))),
    };

    env.get_interface().send_message(
        target_address,
        target_handler,
        validity_start,
        validity_end,
        max_gas as u64,
        raw_fee as u64,
        raw_coins as u64,
        data,
        filter,
    )?;
    Ok(())
}

/// gets the period of the current execution slot
pub(crate) fn assembly_script_get_current_period(env: &ASEnv) -> ABIResult<i64> {
    sub_remaining_gas(env, settings::metering_get_current_period())?;
    Ok(env.get_interface().get_current_period()? as i64)
}

/// gets the thread of the current execution slot
pub(crate) fn assembly_script_get_current_thread(env: &ASEnv) -> ABIResult<i32> {
    sub_remaining_gas(env, settings::metering_get_current_thread())?;
    Ok(env.get_interface().get_current_thread()? as i32)
}

/// sets the executable bytecode of an arbitrary address
#[named]
pub(crate) fn assembly_script_set_bytecode_for(
    env: &ASEnv,
    address: i32,
    bytecode: i32,
) -> ABIResult<()> {
    sub_remaining_gas(env, settings::metering_set_bytecode_const())?;
    let memory = get_memory!(env);
    let address = get_string(memory, address)?;
    let bytecode_raw = read_buffer_and_sub_gas(
        env,
        memory,
        bytecode,
        settings::metering_set_bytecode_mult(),
    )?;
    if cfg!(feature = "gas_calibration") {
        let fname = format!("massa.{}:0", function_name!());
        param_size_update(env, &fname, address.len(), true);
        let fname = format!("massa.{}:1", function_name!());
        param_size_update(env, &fname, bytecode_raw.len(), true);
    }
    env.get_interface()
        .raw_set_bytecode_for(&address, &bytecode_raw)?;
    Ok(())
}

/// sets the executable bytecode of the current address
#[named]
pub(crate) fn assembly_script_set_bytecode(env: &ASEnv, bytecode: i32) -> ABIResult<()> {
    sub_remaining_gas(env, settings::metering_set_bytecode_const())?;
    let memory = get_memory!(env);
    let bytecode_raw = read_buffer_and_sub_gas(
        env,
        memory,
        bytecode,
        settings::metering_set_bytecode_mult(),
    )?;
    if cfg!(feature = "gas_calibration") {
        let fname = format!("massa.{}:0", function_name!());
        param_size_update(env, &fname, bytecode_raw.len(), true);
    }
    env.get_interface().raw_set_bytecode(&bytecode_raw)?;
    Ok(())
}

/// get bytecode of the current address
pub(crate) fn assembly_script_get_bytecode(env: &ASEnv) -> ABIResult<i32> {
    sub_remaining_gas(env, settings::metering_get_bytecode_const())?;
    let data = env.get_interface().raw_get_bytecode()?;
    sub_remaining_gas_with_mult(
        env,
        data.len(),
        settings::metering_get_bytecode_value_mult(),
    )?;
    Ok(pointer_from_bytearray(env, &data)?.offset() as i32)
}

/// get bytecode of the target address
pub(crate) fn assembly_script_get_bytecode_for(env: &ASEnv, address: i32) -> ABIResult<i32> {
    sub_remaining_gas(env, settings::metering_get_bytecode_const())?;
    let memory = get_memory!(env);
    let address = get_string(memory, address)?;
    let data = env.get_interface().raw_get_bytecode_for(&address)?;
    sub_remaining_gas_with_mult(
        env,
        data.len(),
        settings::metering_get_bytecode_value_mult(),
    )?;
    Ok(pointer_from_bytearray(env, &data)?.offset() as i32)
}

/// execute `function` of the given bytecode in the current context
pub(crate) fn assembly_script_local_execution(
    env: &ASEnv,
    bytecode: i32,
    function: i32,
    param: i32,
) -> ABIResult<i32> {
    sub_remaining_gas(env, settings::metering_local_execution_const())?;
    let memory = get_memory!(env);

    let bytecode = &read_buffer(memory, bytecode)?;
    let function = &get_string(memory, function)?;
    let param = &read_buffer(memory, param)?;

    let response = local_call(env, bytecode, function, param)?;
    match BufferPtr::alloc(&response.ret, env.get_wasm_env()) {
        Ok(ret) => Ok(ret.offset() as i32),
        _ => abi_bail!(format!(
            "Cannot allocate response in local call of {}",
            function
        )),
    }
}

/// execute `function` of the bytecode located at `address` in the current context
pub(crate) fn assembly_script_local_call(
    env: &ASEnv,
    address: i32,
    function: i32,
    param: i32,
) -> ABIResult<i32> {
    sub_remaining_gas(env, settings::metering_local_call_const())?;
    let memory = get_memory!(env);

    let address = &get_string(memory, address)?;
    // NOTE: how do we handle metering for those cases?
    sub_remaining_gas(env, settings::metering_get_bytecode_const())?;
    let bytecode = env.get_interface().raw_get_bytecode_for(address)?;
    let function = &get_string(memory, function)?;
    let param = &read_buffer(memory, param)?;

    let response = local_call(env, &bytecode, function, param)?;
    match BufferPtr::alloc(&response.ret, env.get_wasm_env()) {
        Ok(ret) => Ok(ret.offset() as i32),
        _ => abi_bail!(format!(
            "Cannot allocate response in local call of {}",
            function
        )),
    }
}

/// Check whether or not the caller has write access in the current context
pub fn assembly_caller_has_write_access(env: &ASEnv) -> ABIResult<i32> {
    sub_remaining_gas(env, settings::metering_local_call_const())?;
    Ok(env.get_interface().caller_has_write_access()? as i32)
}

/// Check whether or not the given function exists at the given address
pub fn assembly_function_exists(env: &ASEnv, address: i32, function: i32) -> ABIResult<i32> {
    sub_remaining_gas(env, settings::metering_local_call_const())?;
    let memory = get_memory!(env);

    let address = &get_string(memory, address)?;
    let function = &get_string(memory, function)?;
    // NOTE: how do we handle metering for those cases?
    sub_remaining_gas(env, settings::metering_get_bytecode_const())?;
    let bytecode = env.get_interface().raw_get_bytecode_for(address)?;

    let module = get_module(&*env.get_interface(), &bytecode)?;
    // NOTE: is there no other way to do this?
    let instance = create_instance(settings::metering_initial_cost(), &module)?;
    Ok(module.has_function(&instance, function) as i32)
}

/// Tooling, return a StringPtr allocated from a String
fn pointer_from_string(env: &ASEnv, value: &str) -> ABIResult<StringPtr> {
    Ok(*StringPtr::alloc(&value.into(), env.get_wasm_env())?)
}

/// Tooling, return a BufferPtr allocated from bytes
fn pointer_from_bytearray(env: &ASEnv, value: &Vec<u8>) -> ABIResult<BufferPtr> {
    Ok(*BufferPtr::alloc(value, env.get_wasm_env())?)
}

/// Tooling that reads a String in memory and subtract remaining gas
/// with a multiplicator (String.len * mult).
///
/// Sub function of `assembly_script_set_data_for`, `assembly_script_set_data`
/// and `assembly_script_create_sc`
///
/// Return the string value in the StringPtr
fn read_string_and_sub_gas(
    env: &ASEnv,
    memory: &Memory,
    offset: i32,
    mult: usize,
) -> ABIResult<String> {
    let value = StringPtr::new(offset as u32).read(memory)?;
    sub_remaining_gas_with_mult(env, value.len(), mult)?;
    Ok(value)
}

/// Tooling that reads a buffer (Vec<u8>) in memory
fn read_buffer(memory: &Memory, offset: i32) -> ABIResult<Vec<u8>> {
    Ok(BufferPtr::new(offset as u32).read(memory)?)
}

/// Tooling that reads a buffer (Vec<u8>) in memory and subtract remaining gas
/// with a multiplicator (buffer len * mult).
///
/// Return the buffer in the BufferPtr
fn read_buffer_and_sub_gas(
    env: &ASEnv,
    memory: &Memory,
    offset: i32,
    mult: usize,
) -> ABIResult<Vec<u8>> {
    let buffer = BufferPtr::new(offset as u32).read(memory)?;
    sub_remaining_gas_with_mult(env, buffer.len(), mult)?;
    Ok(buffer)
}

/// Tooling, return a string from a given offset
fn get_string(memory: &Memory, ptr: i32) -> ABIResult<String> {
    Ok(StringPtr::new(ptr as u32).read(memory)?)
}

/// Tooling, return a pointer offset of a serialized list in json
fn alloc_string_array(env: &ASEnv, vec: &[String]) -> ABIResult<i32> {
    let addresses = serde_json::to_string(vec)?;
    Ok(StringPtr::alloc(&addresses, env.get_wasm_env())?.offset() as i32)
}

/// Flatten a Vec<Vec<u8>> (or anything that can be turned into an iterator) to a Vec<u8>
/// with the format: L (32 bits LE) V1_L (8 bits) V1 (8bits * V1_L), V2_L ... VN (8 bits * VN_L)
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

#[cfg(test)]
mod tests {
    use crate::execution::as_abi::ser_bytearray_vec;

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
