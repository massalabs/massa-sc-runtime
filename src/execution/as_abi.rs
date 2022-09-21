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
use crate::settings;
use as_ffi_bindings::{BufferPtr, Read as ASRead, StringPtr, Write as ASWrite};
use wasmer::Memory;

use super::common::{abi_bail, call_module, create_sc, ABIResult};

/// Get the coins that have been made available for a specific purpose for the current call.
pub(crate) fn assembly_script_get_call_coins(env: &ASEnv) -> ABIResult<i64> {
    sub_remaining_gas(env, settings::metering_get_call_coins())?;
    match env.get_interface().get_call_coins() {
        Ok(res) => Ok(res as i64),
        Err(err) => abi_bail!(err),
    }
}

/// Transfer an amount from the address on the current call stack to a target address.
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
    match env
        .get_interface()
        .transfer_coins(to_address, raw_amount as u64)
    {
        Ok(res) => Ok(res),
        Err(err) => abi_bail!(err),
    }
}

/// Transfer an amount from the specified address to a target address.
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
    match env
        .get_interface()
        .transfer_coins_for(from_address, to_address, raw_amount as u64)
    {
        Ok(res) => Ok(res),
        Err(err) => abi_bail!(err),
    }
}

pub(crate) fn assembly_script_get_balance(env: &ASEnv) -> ABIResult<i64> {
    sub_remaining_gas(env, settings::metering_get_balance())?;
    match env.get_interface().get_balance() {
        Ok(res) => Ok(res as i64),
        Err(err) => abi_bail!(err),
    }
}

pub(crate) fn assembly_script_get_balance_for(env: &ASEnv, address: i32) -> ABIResult<i64> {
    sub_remaining_gas(env, settings::metering_get_balance())?;
    let memory = get_memory!(env);
    let address = &get_string(memory, address)?;
    match env.get_interface().get_balance_for(address) {
        Ok(res) => Ok(res as i64),
        Err(err) => abi_bail!(err),
    }
}

/// Raw call that have the right type signature to be able to be call a module
/// directly form AssemblyScript:
pub(crate) fn assembly_script_call_module(
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
    let param = &get_string(memory, param)?;
    let response = call_module(env, address, function, param, call_coins)?;
    match StringPtr::alloc(&response.ret, env.get_wasm_env()) {
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
pub(crate) fn assembly_script_print(env: &ASEnv, arg: i32) -> ABIResult<()> {
    sub_remaining_gas(env, settings::metering_print())?;
    let memory = get_memory!(env);
    if let Err(err) = env.get_interface().print(&get_string(memory, arg)?) {
        abi_bail!(err);
    }
    Ok(())
}

/// Get the operation datastore keys (aka entries)
pub(crate) fn assembly_script_get_op_keys(env: &ASEnv) -> ABIResult<i32> {
    match env.get_interface().get_op_keys() {
        Err(err) => abi_bail!(err),
        Ok(k) => {
            sub_remaining_gas_with_mult(
                env,
                k.iter().fold(0, |acc, v_| acc + v_.len()),
                settings::get_op_keys_mult(),
            )?;
            let k_f = ser_bytearray_vec(&k, settings::max_op_datastore_entry_count())?;
            let a = pointer_from_bytearray(env, &k_f)?.offset();
            Ok(a as i32)
        }
    }
}

/// Check if a key is present in operation datastore
pub(crate) fn assembly_script_has_op_key(env: &ASEnv, arg: i32) -> ABIResult<i32> {
    let memory = get_memory!(env);
    let key = read_buffer_and_sub_gas(env, memory, arg, settings::has_op_key_mult())?;
    match env.get_interface().has_op_key(&key) {
        Err(err) => abi_bail!(err),
        Ok(b) => {
            // https://doc.rust-lang.org/reference/types/boolean.html
            // 'true' is explicitly defined as: 0x01 while 'false' is: 0x00
            let b_vec: Vec<u8> = vec![b as u8];
            let a = pointer_from_bytearray(env, &b_vec)?.offset();
            Ok(a as i32)
        }
    }
}

/// Get the operation datastore value associated to given key
pub(crate) fn assembly_script_get_op_data(env: &ASEnv, arg: i32) -> ABIResult<i32> {
    let memory = get_memory!(env);
    let key = read_buffer_and_sub_gas(env, memory, arg, settings::get_op_data_mult())?;
    match env.get_interface().get_op_data(&key) {
        Err(err) => abi_bail!(err),
        Ok(b) => {
            let a = pointer_from_bytearray(env, &b)?.offset();
            Ok(a as i32)
        }
    }
}

/// Read a bytecode string, representing the webassembly module binary encoded
/// with in base64.
pub(crate) fn assembly_script_create_sc(env: &ASEnv, bytecode: i32) -> ABIResult<i32> {
    let memory = get_memory!(env);
    // Base64 to Binary
    let bytecode = match base64::decode(read_string_and_sub_gas(
        env,
        memory,
        bytecode,
        settings::metering_create_sc_mult(),
    )?) {
        Ok(bytecode) => bytecode,
        Err(err) => abi_bail!(err),
    };
    let address = match create_sc(env, &bytecode) {
        Ok(address) => address,
        Err(err) => abi_bail!(err),
    };
    match StringPtr::alloc(&address, env.get_wasm_env()) {
        Ok(ptr) => Ok(ptr.offset() as i32),
        Err(err) => abi_bail!(err),
    }
}

/// performs a hash on a string and returns the bs58check encoded hash
pub(crate) fn assembly_script_hash(env: &ASEnv, value: i32) -> ABIResult<i32> {
    sub_remaining_gas(env, settings::metering_hash_const())?;
    let memory = get_memory!(env);
    let value = read_string_and_sub_gas(env, memory, value, settings::metering_hash_per_byte())?;
    match env.get_interface().hash(value.as_bytes()) {
        Ok(h) => Ok(pointer_from_string(env, &h)?.offset() as i32),
        Err(err) => abi_bail!(err),
    }
}

/// sets a key-indexed data entry in the datastore, overwriting existing values if any
pub(crate) fn assembly_script_set_data(env: &ASEnv, key: i32, value: i32) -> ABIResult<()> {
    sub_remaining_gas(env, settings::metering_set_data_const())?;
    let memory = get_memory!(env);
    let key = read_string_and_sub_gas(env, memory, key, settings::metering_set_data_key_mult())?;
    let value =
        read_string_and_sub_gas(env, memory, value, settings::metering_set_data_value_mult())?;
    if let Err(err) = env.get_interface().raw_set_data(&key, value.as_bytes()) {
        abi_bail!(err)
    }
    Ok(())
}

/// appends data to a key-indexed data entry in the datastore, fails if the entry does not exist
pub(crate) fn assembly_script_append_data(env: &ASEnv, key: i32, value: i32) -> ABIResult<()> {
    sub_remaining_gas(env, settings::metering_append_data_const())?;
    let memory = get_memory!(env);
    let key = read_string_and_sub_gas(env, memory, key, settings::metering_append_data_key_mult())?;
    let value = read_string_and_sub_gas(
        env,
        memory,
        value,
        settings::metering_append_data_value_mult(),
    )?;
    if let Err(err) = env.get_interface().raw_append_data(&key, value.as_bytes()) {
        abi_bail!(err)
    }
    Ok(())
}

/// gets a key-indexed data entry in the datastore, failing if non-existent
pub(crate) fn assembly_script_get_data(env: &ASEnv, key: i32) -> ABIResult<i32> {
    sub_remaining_gas(env, settings::metering_get_data_const())?;
    let memory = get_memory!(env);
    let key = read_string_and_sub_gas(env, memory, key, settings::metering_get_data_key_mult())?;
    match env.get_interface().raw_get_data(&key) {
        Ok(data) => {
            sub_remaining_gas_with_mult(env, data.len(), settings::metering_get_data_value_mult())?;
            Ok(pointer_from_utf8(env, &data)?.offset() as i32)
        }
        Err(err) => abi_bail!(err),
    }
}

/// checks if a key-indexed data entry exists in the datastore
pub(crate) fn assembly_script_has_data(env: &ASEnv, key: i32) -> ABIResult<i32> {
    sub_remaining_gas(env, settings::metering_has_data_const())?;
    let memory = get_memory!(env);
    let key = read_string_and_sub_gas(env, memory, key, settings::metering_has_data_key_mult())?;
    match env.get_interface().has_data(&key) {
        Ok(true) => Ok(1),
        Ok(false) => Ok(0),
        Err(err) => abi_bail!(err),
    }
}

/// deletes a key-indexed data entry in the datastore of the current address, fails if the entry is absent
pub(crate) fn assembly_script_delete_data(env: &ASEnv, key: i32) -> ABIResult<()> {
    sub_remaining_gas(env, settings::metering_delete_data_const())?;
    let memory = get_memory!(env);
    let key = read_string_and_sub_gas(env, memory, key, settings::metering_delete_data_key_mult())?;
    match env.get_interface().raw_delete_data(&key) {
        Ok(_) => Ok(()),
        Err(err) => abi_bail!(err),
    }
}

/// Sets the value of a datastore entry of an arbitrary address, creating the entry if it does not exist.
/// Fails if the address does not exist.
pub(crate) fn assembly_script_set_data_for(
    env: &ASEnv,
    address: i32,
    key: i32,
    value: i32,
) -> ABIResult<()> {
    sub_remaining_gas(env, settings::metering_set_data_const())?;
    let memory = get_memory!(env);
    let key = read_string_and_sub_gas(env, memory, key, settings::metering_set_data_key_mult())?;
    let value =
        read_string_and_sub_gas(env, memory, value, settings::metering_set_data_value_mult())?;
    let address = get_string(memory, address)?;
    if let Err(err) = env
        .get_interface()
        .raw_set_data_for(&address, &key, value.as_bytes())
    {
        abi_bail!(err)
    }
    Ok(())
}

/// Appends data to the value of a datastore entry of an arbitrary address, fails if the entry or address does not exist.
pub(crate) fn assembly_script_append_data_for(
    env: &ASEnv,
    address: i32,
    key: i32,
    value: i32,
) -> ABIResult<()> {
    sub_remaining_gas(env, settings::metering_append_data_const())?;
    let memory = get_memory!(env);
    let key = read_string_and_sub_gas(env, memory, key, settings::metering_append_data_key_mult())?;
    let value = read_string_and_sub_gas(
        env,
        memory,
        value,
        settings::metering_append_data_value_mult(),
    )?;
    let address = get_string(memory, address)?;
    if let Err(err) = env
        .get_interface()
        .raw_append_data_for(&address, &key, value.as_bytes())
    {
        abi_bail!(err)
    }
    Ok(())
}

/// Gets the value of a datastore entry for an arbitrary address, fails if the entry or address does not exist
pub(crate) fn assembly_script_get_data_for(env: &ASEnv, address: i32, key: i32) -> ABIResult<i32> {
    sub_remaining_gas(env, settings::metering_get_data_const())?;
    let memory = get_memory!(env);
    let address = get_string(memory, address)?;
    let key = read_string_and_sub_gas(env, memory, key, settings::metering_get_data_key_mult())?;
    match env.get_interface().raw_get_data_for(&address, &key) {
        Ok(data) => {
            sub_remaining_gas_with_mult(env, data.len(), settings::metering_get_data_value_mult())?;
            Ok(pointer_from_utf8(env, &data)?.offset() as i32)
        }
        Err(err) => abi_bail!(err),
    }
}

/// Deletes a datastore entry for an address. Fails if the entry or address does not exist.
pub(crate) fn assembly_script_delete_data_for(
    env: &ASEnv,
    address: i32,
    key: i32,
) -> ABIResult<()> {
    sub_remaining_gas(env, settings::metering_delete_data_const())?;
    let memory = get_memory!(env);
    let address = get_string(memory, address)?;
    let key = read_string_and_sub_gas(env, memory, key, settings::metering_delete_data_key_mult())?;
    match env.get_interface().raw_delete_data_for(&address, &key) {
        Ok(_) => Ok(()),
        Err(err) => abi_bail!(err),
    }
}

pub(crate) fn assembly_script_has_data_for(env: &ASEnv, address: i32, key: i32) -> ABIResult<i32> {
    sub_remaining_gas(env, settings::metering_has_data_const())?;
    let memory = get_memory!(env);
    let address = get_string(memory, address)?;
    let key = read_string_and_sub_gas(env, memory, key, settings::metering_has_data_key_mult())?;
    match env.get_interface().has_data_for(&address, &key) {
        Ok(true) => Ok(1),
        Ok(false) => Ok(0),
        Err(err) => abi_bail!(err),
    }
}

pub(crate) fn assembly_script_get_owned_addresses_raw(env: &ASEnv) -> ABIResult<i32> {
    sub_remaining_gas(env, settings::metering_get_owned_addrs())?;
    let data = match env.get_interface().get_owned_addresses() {
        Ok(data) => data,
        Err(err) => abi_bail!(err),
    };
    match StringPtr::alloc(&data.join(";"), env.get_wasm_env()) {
        Ok(ptr) => Ok(ptr.offset() as i32),
        Err(err) => abi_bail!(err),
    }
}

pub(crate) fn assembly_script_get_call_stack_raw(env: &ASEnv) -> ABIResult<i32> {
    sub_remaining_gas(env, settings::metering_get_call_stack())?;
    let data = match env.get_interface().get_call_stack() {
        Ok(data) => data,
        Err(err) => abi_bail!(err),
    };
    match StringPtr::alloc(&data.join(";"), env.get_wasm_env()) {
        Ok(ptr) => Ok(ptr.offset() as i32),
        Err(err) => abi_bail!(err),
    }
}

pub(crate) fn assembly_script_get_owned_addresses(env: &ASEnv) -> ABIResult<i32> {
    sub_remaining_gas(env, settings::metering_get_owned_addrs())?;
    match env.get_interface().get_owned_addresses() {
        Ok(data) => alloc_string_array(env, &data),
        Err(err) => abi_bail!(err),
    }
}

pub(crate) fn assembly_script_get_call_stack(env: &ASEnv) -> ABIResult<i32> {
    sub_remaining_gas(env, settings::metering_get_call_stack())?;
    match env.get_interface().get_call_stack() {
        Ok(data) => alloc_string_array(env, &data),
        Err(err) => abi_bail!(err),
    }
}

pub(crate) fn assembly_script_generate_event(env: &ASEnv, event: i32) -> ABIResult<()> {
    sub_remaining_gas(env, settings::metering_generate_event())?;
    let memory = get_memory!(env);
    let event = get_string(memory, event)?;
    if let Err(err) = env.get_interface().generate_event(event) {
        abi_bail!(err)
    }
    Ok(())
}

/// verify a signature of data given a public key. Returns Ok(1) if correctly verified, otherwise Ok(0)
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
    match env
        .get_interface()
        .signature_verify(data.as_bytes(), &signature, &public_key)
    {
        Err(err) => abi_bail!(err),
        Ok(false) => Ok(0),
        Ok(true) => Ok(1),
    }
}

/// converts a public key to an address
pub(crate) fn assembly_script_address_from_public_key(
    env: &ASEnv,
    public_key: i32,
) -> ABIResult<i32> {
    sub_remaining_gas(env, settings::metering_address_from_public_key())?;
    let memory = get_memory!(env);
    let public_key = get_string(memory, public_key)?;
    match env.get_interface().address_from_public_key(&public_key) {
        Err(err) => abi_bail!(err),
        Ok(addr) => Ok(pointer_from_string(env, &addr)?.offset() as i32),
    }
}

/// generates an unsafe random number
pub(crate) fn assembly_script_unsafe_random(env: &ASEnv) -> ABIResult<i64> {
    sub_remaining_gas(env, settings::metering_unsafe_random())?;
    match env.get_interface().unsafe_random() {
        Err(err) => abi_bail!(err),
        Ok(rnd) => Ok(rnd),
    }
}

/// gets the current unix timestamp in milliseconds
pub(crate) fn assembly_script_get_time(env: &ASEnv) -> ABIResult<i64> {
    sub_remaining_gas(env, settings::metering_get_time())?;
    match env.get_interface().get_time() {
        Err(err) => abi_bail!(err),
        Ok(t) => Ok(t as i64),
    }
}

/// sends an async message
#[allow(clippy::too_many_arguments)]
pub(crate) fn assembly_script_send_message(
    env: &ASEnv,
    target_address: i32,
    target_handler: i32,
    validity_start_period: i64,
    validity_start_thread: i32,
    validity_end_period: i64,
    validity_end_thread: i32,
    max_gas: i64,
    gas_price: i64,
    raw_coins: i64,
    data: i32,
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
    if gas_price.is_negative() {
        abi_bail!("negative gas price");
    }
    if raw_coins.is_negative() {
        abi_bail!("negative coins")
    }
    let memory = get_memory!(env);
    match env.get_interface().send_message(
        &get_string(memory, target_address)?,
        &get_string(memory, target_handler)?,
        validity_start,
        validity_end,
        max_gas as u64,
        gas_price as u64,
        raw_coins as u64,
        get_string(memory, data)?.as_bytes(),
    ) {
        Err(err) => abi_bail!(err),
        Ok(_) => Ok(()),
    }
}

/// gets the period of the current execution slot
pub(crate) fn assembly_script_get_current_period(env: &ASEnv) -> ABIResult<i64> {
    sub_remaining_gas(env, settings::metering_get_current_period())?;
    match env.get_interface().get_current_period() {
        Err(err) => abi_bail!(err),
        Ok(v) => Ok(v as i64),
    }
}

/// gets the thread of the current execution slot
pub(crate) fn assembly_script_get_current_thread(env: &ASEnv) -> ABIResult<i32> {
    sub_remaining_gas(env, settings::metering_get_current_thread())?;
    match env.get_interface().get_current_thread() {
        Err(err) => abi_bail!(err),
        Ok(v) => Ok(v as i32),
    }
}

/// sets the executable bytecode of an arbitrary address
pub(crate) fn assembly_script_set_bytecode_for(
    env: &ASEnv,
    address: i32,
    bytecode_base64: i32,
) -> ABIResult<()> {
    sub_remaining_gas(env, settings::metering_set_bytecode_const())?;
    let memory = get_memory!(env);
    let address = get_string(memory, address)?;
    let bytecode_base64 = read_string_and_sub_gas(
        env,
        memory,
        bytecode_base64,
        settings::metering_set_bytecode_mult(),
    )?;
    let bytecode_raw = match base64::decode(bytecode_base64) {
        Ok(v) => v,
        Err(err) => abi_bail!(err),
    };
    match env
        .get_interface()
        .raw_set_bytecode_for(&address, &bytecode_raw)
    {
        Ok(()) => Ok(()),
        Err(err) => abi_bail!(err),
    }
}

/// sets the executable bytecode of the current address
pub(crate) fn assembly_script_set_bytecode(env: &ASEnv, bytecode_base64: i32) -> ABIResult<()> {
    sub_remaining_gas(env, settings::metering_set_bytecode_const())?;
    let memory = get_memory!(env);
    let bytecode_base64 = read_string_and_sub_gas(
        env,
        memory,
        bytecode_base64,
        settings::metering_set_bytecode_mult(),
    )?;
    let bytecode_raw = match base64::decode(bytecode_base64) {
        Ok(v) => v,
        Err(err) => abi_bail!(err),
    };
    match env.get_interface().raw_set_bytecode(&bytecode_raw) {
        Ok(()) => Ok(()),
        Err(err) => abi_bail!(err),
    }
}

/// Tooling, return a StringPtr allocated from a String
fn pointer_from_string(env: &ASEnv, value: &str) -> ABIResult<StringPtr> {
    match StringPtr::alloc(&value.into(), env.get_wasm_env()) {
        Ok(ptr) => Ok(*ptr),
        Err(err) => abi_bail!(err),
    }
}

/// Tooling, return a StringPtr allocated from bytes with utf8 parsing
fn pointer_from_utf8(env: &ASEnv, value: &[u8]) -> ABIResult<StringPtr> {
    match std::str::from_utf8(value) {
        Ok(data) => match StringPtr::alloc(&data.to_string(), env.get_wasm_env()) {
            Ok(ptr) => Ok(*ptr),
            Err(err) => abi_bail!(err),
        },
        Err(err) => abi_bail!(err),
    }
}

/// Tooling, return a BufferPtr allocated from bytes
fn pointer_from_bytearray(env: &ASEnv, value: &Vec<u8>) -> ABIResult<BufferPtr> {
    match BufferPtr::alloc(value, env.get_wasm_env()) {
        Ok(ptr) => Ok(*ptr),
        Err(e) => abi_bail!(e),
    }
}

/// Tooling that take read a String in memory and subtract remaining gas
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
    match StringPtr::new(offset as u32).read(memory) {
        Ok(value) => {
            sub_remaining_gas_with_mult(env, value.len(), mult)?;
            Ok(value)
        }
        Err(err) => abi_bail!(err),
    }
}

/// Tooling that take read a buffer (Vec<u8>) in memory and subtract remaining gas
/// with a multiplicator (buffer len * mult).
///
/// Return the buffer in the BufferPtr
fn read_buffer_and_sub_gas(
    env: &ASEnv,
    memory: &Memory,
    offset: i32,
    mult: usize,
) -> ABIResult<Vec<u8>> {
    match BufferPtr::new(offset as u32).read(memory) {
        Ok(buffer) => {
            sub_remaining_gas_with_mult(env, buffer.len(), mult)?;
            Ok(buffer)
        }
        Err(err) => abi_bail!(err),
    }
}

/// Tooling, return a string from a given offset
fn get_string(memory: &Memory, ptr: i32) -> ABIResult<String> {
    match StringPtr::new(ptr as u32).read(memory) {
        Ok(str) => Ok(str),
        Err(err) => abi_bail!(err),
    }
}

/// Tooling, return a pointer offset of a serialized list in json
fn alloc_string_array(env: &ASEnv, vec: &[String]) -> ABIResult<i32> {
    let addresses = match serde_json::to_string(vec) {
        Ok(list) => list,
        Err(err) => abi_bail!(err),
    };
    match StringPtr::alloc(&addresses, env.get_wasm_env()) {
        Ok(ptr) => Ok(ptr.offset() as i32),
        Err(err) => abi_bail!(err),
    }
}

#[allow(dead_code)]
/// Tooling, return a buffer (Vec<u8>) from a given offset
fn get_buffer(memory: &Memory, ptr: i32) -> ABIResult<Vec<u8>> {
    match BufferPtr::new(ptr as u32).read(memory) {
        Ok(buffer) => Ok(buffer),
        Err(err) => abi_bail!(err),
    }
}

/// Flatten a Vec<Vec<u8>> to a Vec<u8> with the format:
/// L (32 bits LE) V1_L (8 bits) V1 (8bits * V1_L), V2_L ... VN (8 bits * VN_L)
fn ser_bytearray_vec(data: &Vec<Vec<u8>>, max_entry: usize) -> ABIResult<Vec<u8>> {
    if data.is_empty() {
        return Ok(Vec::new());
    }

    // u16::MAX is still ok; u32::MAX -> panic!
    if data.len() > max_entry || data.len() > u16::MAX as usize {
        abi_bail!("Too many entries in the datastore");
    }

    // pre alloc with max capacity
    let mut buffer = Vec::with_capacity(4 + (data.len() as usize * (1+255)));

    let entry_count = u32::try_from(data.len()).unwrap();
    buffer.extend_from_slice(&entry_count.to_le_bytes());

    for key in data.iter() {
        let k_len = match u8::try_from(key.len()) {
            Ok(l) => l,
            Err(_) => abi_bail!("Some Datastore key are too long"),
        };
        buffer.push(k_len);
        buffer.extend_from_slice(key);

    }

    return Ok(buffer);
}

#[cfg(test)]
mod tests {
    use crate::execution::as_abi::ser_bytearray_vec;

    #[test]
    fn test_ser() {
        let vb: Vec<Vec<u8>> = vec![vec![1, 2, 3], vec![255]];

        let vb_ser = ser_bytearray_vec(&vb, 10).unwrap();
        assert_eq!(vb_ser, [2, 0, 0, 0, 3, 1, 2, 3, 1, 255]);
    }

    #[test]
    fn test_ser_edge_cases() {
        // FIXME: should we support theses edge cases or bail?

        let vb: Vec<Vec<u8>> = vec![vec![1, 2, 3], vec![]];

        let vb_ser = ser_bytearray_vec(&vb, 10).unwrap();
        assert_eq!(vb_ser, [2, 0, 0, 0, 3, 1, 2, 3, 0]);

        let vb_ser = ser_bytearray_vec(&vb, 1);
        assert!(vb_ser.is_err());

        let vb: Vec<Vec<u8>> = vec![];
        let vb_ser = ser_bytearray_vec(&vb, 10).unwrap();
        let empty_vec: Vec<u8> = vec![];
        assert_eq!(vb_ser, empty_vec);

        // A really big vec to serialize
        let vb: Vec<Vec<u8>> = (0..=u8::MAX)
            .cycle()
            .take(u16::MAX as usize)
            .map(|i| vec![i])
            .collect();
        assert_eq!(vb.len(), u16::MAX as usize);

        let vb_ser = ser_bytearray_vec(&vb, u16::MAX as usize).unwrap();
        assert_eq!(vb_ser[0..4], [255, 255, 0, 0]);
        assert_eq!(vb_ser[4], 1);
        assert_eq!(vb_ser[4 + 1], 0);
        assert_eq!(vb_ser[4 + 2], 1);
        assert_eq!(vb_ser[4 + 3], 1);
        assert_eq!(vb_ser[vb_ser.len() - 2], 1);
        assert_eq!(vb_ser[vb_ser.len() - 1], 254);
    }
}
