///! *abi_impl.rs* contains all the implementation (and some tools as
///! abi_bail!) of the massa abi.
///!
///! The ABIs are the imported function / object declared in the webassembly
///! module. You can look at the other side of the mirror in `massa.ts` and the
///! rust side in `execution_impl.rs`.
///!
///! ```
use crate::env::{
    get_remaining_points_for_env, sub_remaining_gas, sub_remaining_gas_with_mult, Env,
};
use crate::types::{Address, Response};
use crate::{settings, Bytecode};
use as_ffi_bindings::{Read as ASRead, StringPtr, Write as ASWrite};
use wasmer::Memory;

pub type ABIResult<T, E = wasmer::RuntimeError> = core::result::Result<T, E>;
macro_rules! abi_bail {
    ($err:expr) => {
        return Err(wasmer::RuntimeError::new($err.to_string()))
    };
}
macro_rules! get_memory {
    ($env:ident) => {
        match $env.wasm_env.memory.get_ref() {
            Some(mem) => mem,
            _ => abi_bail!("uninitialized memory"),
        }
    };
}
pub(crate) use abi_bail;
pub(crate) use get_memory;

/// `Call` ABI called by the webassembly VM
///
/// Call an exported function in a WASM module at a given address
///
/// It take in argument the environment defined in env.rs
/// this environment is automatically filled by the wasmer library
/// And two pointers of string. (look at the readme in the wasm folder)
fn call_module(env: &Env, address: &Address, function: &str, param: &str) -> ABIResult<Response> {
    let module = &match env.interface.get_module(address) {
        Ok(module) => module,
        Err(err) => abi_bail!(err),
    };
    let res = match crate::execution_impl::exec(
        get_remaining_points_for_env(env),
        None,
        module,
        function,
        param,
        &*env.interface,
    ) {
        Ok(response) => response,
        Err(err) => abi_bail!(err),
    };
    match env.interface.exit_success() {
        Ok(_) => Ok(res),
        Err(err) => abi_bail!(err),
    }
}

/// Transfer an amount from the address on the current call stack to a target address.
pub(crate) fn assembly_script_transfer_coins(
    env: &Env,
    to_address: i32,
    raw_amount: i64,
) -> ABIResult<()> {
    sub_remaining_gas(env, settings::metering_transfer())?;
    if raw_amount.is_negative() {
        abi_bail!("Negative raw amount.");
    }
    let memory = get_memory!(env);
    let to_address = &get_string(memory, to_address)?;
    match env.interface.transfer_coins(to_address, raw_amount as u64) {
        Ok(res) => Ok(res),
        Err(err) => abi_bail!(err),
    }
}

/// Transfer an amount from the specified address to a target address.
pub(crate) fn assembly_script_transfer_coins_for(
    env: &Env,
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
        .interface
        .transfer_coins_for(from_address, to_address, raw_amount as u64)
    {
        Ok(res) => Ok(res),
        Err(err) => abi_bail!(err),
    }
}

pub(crate) fn assembly_script_get_balance(env: &Env) -> ABIResult<i64> {
    sub_remaining_gas(env, settings::metering_get_balance())?;
    match env.interface.get_balance() {
        Ok(res) => Ok(res as i64),
        Err(err) => abi_bail!(err),
    }
}

pub(crate) fn assembly_script_get_balance_for(env: &Env, address: i32) -> ABIResult<i64> {
    sub_remaining_gas(env, settings::metering_get_balance())?;
    let memory = get_memory!(env);
    let address = &get_string(memory, address)?;
    match env.interface.get_balance_for(address) {
        Ok(res) => Ok(res as i64),
        Err(err) => abi_bail!(err),
    }
}

fn create_sc(env: &Env, bytecode: &Bytecode) -> ABIResult<Address> {
    match env.interface.create_module(bytecode) {
        Ok(address) => Ok(address),
        Err(err) => abi_bail!(err),
    }
}

/// Raw call that have the right type signature to be able to be call a module
/// directly form AssemblyScript:
#[doc = include_str!("../wasm/README.md")]
pub(crate) fn assembly_script_call_module(
    env: &Env,
    address: i32,
    function: i32,
    param: i32,
) -> ABIResult<i32> {
    sub_remaining_gas(env, settings::metering_call())?;
    let memory = get_memory!(env);
    let address = &get_string(memory, address)?;
    let function = &get_string(memory, function)?;
    let param = &get_string(memory, param)?;
    let response = call_module(env, address, function, param)?;
    match StringPtr::alloc(&response.ret, &env.wasm_env) {
        Ok(ret) => Ok(ret.offset() as i32),
        _ => abi_bail!(format!(
            "Cannot allocate response in call {}::{}",
            address, function
        )),
    }
}

pub(crate) fn assembly_script_get_remaining_gas(env: &Env) -> ABIResult<i64> {
    sub_remaining_gas(env, settings::metering_remaining_gas())?;
    Ok(get_remaining_points_for_env(env) as i64)
}

/// Create an instance of VM from a module with a
/// given intefrace, an operation number limit and a webassembly module
///
/// An utility print function to write on stdout directly from AssemblyScript:
pub(crate) fn assembly_script_print(env: &Env, arg: i32) -> ABIResult<()> {
    sub_remaining_gas(env, settings::metering_print())?;
    let memory = get_memory!(env);
    if let Err(err) = env.interface.print(&get_string(memory, arg)?) {
        abi_bail!(err);
    }
    Ok(())
}

/// Read a bytecode string, representing the webassembly module binary encoded
/// with in base64.
pub(crate) fn assembly_script_create_sc(env: &Env, bytecode: i32) -> ABIResult<i32> {
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
    match StringPtr::alloc(&address, &env.wasm_env) {
        Ok(ptr) => Ok(ptr.offset() as i32),
        Err(err) => abi_bail!(err),
    }
}

/// performs a hash on a string and returns the bs58check encoded hash
pub(crate) fn assembly_script_hash(env: &Env, value: i32) -> ABIResult<i32> {
    sub_remaining_gas(env, settings::metering_hash_const())?;
    let memory = get_memory!(env);
    let value = read_string_and_sub_gas(env, memory, value, settings::metering_hash_per_byte())?;
    match env.interface.hash(&value.as_bytes().to_vec()) {
        Ok(h) => Ok(pointer_from_string(env, &h)?.offset() as i32),
        Err(err) => abi_bail!(err),
    }
}

/// sets a key-indexed data entry in the datastore, overwriting existing values if any
pub(crate) fn assembly_script_set_data(env: &Env, key: i32, value: i32) -> ABIResult<()> {
    sub_remaining_gas(env, settings::metering_set_data_const())?;
    let memory = get_memory!(env);
    let key = read_string_and_sub_gas(env, memory, key, settings::metering_set_data_key_mult())?;
    let value =
        read_string_and_sub_gas(env, memory, value, settings::metering_set_data_value_mult())?;
    if let Err(err) = env.interface.set_data(&key, &value.as_bytes().to_vec()) {
        abi_bail!(err)
    }
    Ok(())
}

/// gets a key-indexed data entry in the datastore, failing if non-existant
pub(crate) fn assembly_script_get_data(env: &Env, key: i32) -> ABIResult<i32> {
    sub_remaining_gas(env, settings::metering_get_data_const())?;
    let memory = get_memory!(env);
    let key = read_string_and_sub_gas(env, memory, key, settings::metering_get_data_key_mult())?;
    match env.interface.get_data(&key) {
        Ok(data) => {
            sub_remaining_gas_with_mult(env, data.len(), settings::metering_get_data_value_mult())?;
            Ok(pointer_from_utf8(env, &data)?.offset() as i32)
        }
        Err(err) => abi_bail!(err),
    }
}

/// checks if a key-indexed data entry exists in the datastore
pub(crate) fn assembly_script_has_data(env: &Env, key: i32) -> ABIResult<i32> {
    sub_remaining_gas(env, settings::metering_has_data_const())?;
    let memory = get_memory!(env);
    let key = read_string_and_sub_gas(env, memory, key, settings::metering_has_data_key_mult())?;
    match env.interface.has_data(&key) {
        Ok(true) => Ok(1),
        Ok(false) => Ok(0),
        Err(err) => abi_bail!(err),
    }
}

pub(crate) fn assembly_script_set_data_for(
    env: &Env,
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
        .interface
        .set_data_for(&address, &key, &value.as_bytes().to_vec())
    {
        abi_bail!(err)
    }
    Ok(())
}

pub(crate) fn assembly_script_get_data_for(env: &Env, address: i32, key: i32) -> ABIResult<i32> {
    sub_remaining_gas(env, settings::metering_get_data_const())?;
    let memory = get_memory!(env);
    let address = get_string(memory, address)?;
    let key = read_string_and_sub_gas(env, memory, key, settings::metering_get_data_key_mult())?;
    match env.interface.get_data_for(&address, &key) {
        Ok(data) => {
            sub_remaining_gas_with_mult(env, data.len(), settings::metering_get_data_value_mult())?;
            Ok(pointer_from_utf8(env, &data)?.offset() as i32)
        }
        Err(err) => abi_bail!(err),
    }
}

pub(crate) fn assembly_script_has_data_for(env: &Env, address: i32, key: i32) -> ABIResult<i32> {
    sub_remaining_gas(env, settings::metering_has_data_const())?;
    let memory = get_memory!(env);
    let address = get_string(memory, address)?;
    let key = read_string_and_sub_gas(env, memory, key, settings::metering_has_data_key_mult())?;
    match env.interface.has_data_for(&address, &key) {
        Ok(true) => Ok(1),
        Ok(false) => Ok(0),
        Err(err) => abi_bail!(err),
    }
}

pub(crate) fn assembly_script_get_owned_addresses(env: &Env) -> ABIResult<i32> {
    sub_remaining_gas(env, settings::metering_get_owned_addrs())?;
    match env.interface.get_owned_addresses() {
        Ok(data) => alloc_string_array(env, &data),
        Err(err) => abi_bail!(err),
    }
}

pub(crate) fn assembly_script_get_call_stack(env: &Env) -> ABIResult<i32> {
    sub_remaining_gas(env, settings::metering_get_call_stack())?;
    match env.interface.get_call_stack() {
        Ok(data) => alloc_string_array(env, &data),
        Err(err) => abi_bail!(err),
    }
}

pub(crate) fn assembly_script_generate_event(env: &Env, event: i32) -> ABIResult<()> {
    sub_remaining_gas(env, settings::metering_generate_event())?;
    let memory = get_memory!(env);
    let event = get_string(memory, event)?;
    if let Err(err) = env.interface.generate_event(event) {
        abi_bail!(err)
    }
    Ok(())
}

/// verify a signature of data given a public key. Returns Ok(1) if correctly verified, otherwise Ok(0)
pub(crate) fn assembly_script_signature_verify(
    env: &Env,
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
        .interface
        .signature_verify(&data.as_bytes().to_vec(), &signature, &public_key)
    {
        Err(err) => abi_bail!(err),
        Ok(false) => Ok(0),
        Ok(true) => Ok(1),
    }
}

/// converts a public key to an address
pub(crate) fn assembly_script_address_from_public_key(
    env: &Env,
    public_key: i32,
) -> ABIResult<i32> {
    sub_remaining_gas(env, settings::metering_address_from_public_key())?;
    let memory = get_memory!(env);
    let public_key = get_string(memory, public_key)?;
    match env.interface.address_from_public_key(&public_key) {
        Err(err) => abi_bail!(err),
        Ok(addr) => Ok(pointer_from_string(env, &addr)?.offset() as i32),
    }
}

/// generates an unsafe random number
pub(crate) fn assembly_script_unsafe_random(env: &Env) -> ABIResult<i64> {
    sub_remaining_gas(env, settings::metering_unsafe_random())?;
    match env.interface.unsafe_random() {
        Err(err) => abi_bail!(err),
        Ok(rnd) => Ok(rnd),
    }
}

/// gets the current unix timestamp in milliseconds
pub(crate) fn assembly_script_get_time(env: &Env) -> ABIResult<i64> {
    sub_remaining_gas(env, settings::metering_get_time())?;
    match env.interface.get_time() {
        Err(err) => abi_bail!(err),
        Ok(t) => Ok(t as i64),
    }
}

/// Tooling, return a StringPtr allocated from a String
fn pointer_from_string(env: &Env, value: &String) -> ABIResult<StringPtr> {
    match StringPtr::alloc(value, &env.wasm_env) {
        Ok(ptr) => Ok(*ptr),
        Err(err) => abi_bail!(err),
    }
}

/// Tooling, return a StringPtr allocated from bytes with utf8 parsing
fn pointer_from_utf8(env: &Env, value: &Vec<u8>) -> ABIResult<StringPtr> {
    match std::str::from_utf8(value) {
        Ok(data) => match StringPtr::alloc(&data.to_string(), &env.wasm_env) {
            Ok(ptr) => Ok(*ptr),
            Err(err) => abi_bail!(err),
        },
        Err(err) => abi_bail!(err),
    }
}

/// Tooling that take read a String in memory and substract remaining gas
/// with a multiplicator (String.len * mult).
///
/// Sub funtion of `assembly_script_set_data_for`, `assembly_script_set_data`
/// and `assembly_script_create_sc`
///
/// Return the string value in the StringPtr
fn read_string_and_sub_gas(
    env: &Env,
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

/// Tooling, return a string from a given offset
fn get_string(memory: &Memory, ptr: i32) -> ABIResult<String> {
    match StringPtr::new(ptr as u32).read(memory) {
        Ok(str) => Ok(str),
        Err(err) => abi_bail!(err),
    }
}

/// Tooling, return a pointer offset of a serialized list in json
fn alloc_string_array(env: &Env, vec: &[String]) -> ABIResult<i32> {
    let addresses = match serde_json::to_string(vec) {
        Ok(list) => list,
        Err(err) => abi_bail!(err),
    };
    match StringPtr::alloc(&addresses, &env.wasm_env) {
        Ok(ptr) => Ok(ptr.offset() as i32),
        Err(err) => abi_bail!(err),
    }
}
