use cornetto::Cornetto;

// main function name in the webassembly module
pub(crate) const MAIN: &str = "main";

// Metering private implementation
#[allow(dead_code)]
#[derive(Cornetto)]
struct Metering {
    #[cornetto(mut, 200)]
    pub call: u64,
    #[cornetto(const, 200)]
    pub generate_event: u64,
    #[cornetto(const, 200)]
    pub transfer: u64,
    #[cornetto(const, 200)]
    pub get_balance: u64,
    #[cornetto(const, 100)]
    pub set_data_const: u64,
    #[cornetto(const, 1)]
    pub set_data_key_mult: usize,
    #[cornetto(const, 1)]
    pub set_data_value_mult: usize,
    #[cornetto(const, 100)]
    pub get_data_const: u64,
    #[cornetto(const, 1)]
    pub get_data_key_mult: usize,
    #[cornetto(const, 1)]
    pub get_data_value_mult: usize,
    #[cornetto(const, 100)]
    pub delete_data_const: u64,
    #[cornetto(const, 1)]
    pub delete_data_key_mult: usize,
    #[cornetto(const, 100)]
    pub append_data_const: u64,
    #[cornetto(const, 1)]
    pub append_data_key_mult: usize,
    #[cornetto(const, 1)]
    pub append_data_value_mult: usize,
    #[cornetto(const, 100)]
    pub has_data_const: u64,
    #[cornetto(const, 1)]
    pub has_data_key_mult: usize,
    // TODO: define
    #[cornetto(const, 42)]
    pub local_execution_const: u64,
    // TODO: define
    #[cornetto(const, 42)]
    pub local_call_const: u64,
    // TODO: define
    #[cornetto(const, 42)]
    pub get_bytecode_const: u64,
    // TODO: define
    #[cornetto(const, 42)]
    pub get_bytecode_value_mult: usize,
    #[cornetto(const, 1)]
    pub create_sc_mult: usize,
    #[cornetto(const, 200)]
    pub print: u64,
    #[cornetto(const, 200)]
    pub remaining_gas: u64,
    #[cornetto(const, 100)]
    pub get_hash_const: u64,
    #[cornetto(const, 1)]
    pub hash_per_byte: usize,
    #[cornetto(const, 200)]
    pub get_owned_addrs: u64,
    #[cornetto(const, 200)]
    pub get_call_stack: u64,
    #[cornetto(const, 100)]
    pub signature_verify_const: u64,
    #[cornetto(const, 1)]
    pub signature_verify_data_mult: usize,
    #[cornetto(const, 100)]
    pub address_from_public_key: u64,
    #[cornetto(const, 100)]
    pub unsafe_random: u64,
    #[cornetto(const, 100)]
    pub get_time: u64,
    #[cornetto(const, 100)]
    pub get_call_coins: u64,
    #[cornetto(const, 50)]
    pub get_current_period: u64,
    #[cornetto(const, 50)]
    pub get_current_thread: u64,
    #[cornetto(const, 64)]
    pub max_number_of_pages: u32,
    #[cornetto(const, 100)]
    pub send_message: u64,
    #[cornetto(const, 1)]
    pub set_bytecode_mult: usize,
    #[cornetto(const, 100)]
    pub set_bytecode_const: u64,
    #[cornetto(mut, 100)]
    pub initial_cost: u64,
    #[cornetto(const, 128)]
    pub max_op_datastore_entry_count: usize,
    #[cornetto(const, 1)]
    pub has_op_key_mult: usize,
    #[cornetto(const, 1)]
    pub get_op_data_mult: usize,
    #[cornetto(const, 1)]
    pub get_op_keys_mult: usize,
    #[cornetto(const, 1)]
    pub get_keys_mult: usize,
    #[cornetto(const, 100_000)]
    pub max_datastore_entry_count: usize,
}

pub(crate) fn metering_call() -> u64 {
    METERING.call()
}

pub(crate) fn metering_get_call_coins() -> u64 {
    METERING.get_call_coins()
}

pub(crate) fn metering_generate_event() -> u64 {
    METERING.generate_event()
}

pub(crate) fn metering_transfer() -> u64 {
    METERING.transfer()
}

pub(crate) fn metering_get_balance() -> u64 {
    METERING.get_balance()
}

pub(crate) fn metering_print() -> u64 {
    METERING.print()
}

pub(crate) fn metering_create_sc_mult() -> usize {
    METERING.create_sc_mult()
}

pub(crate) fn metering_remaining_gas() -> u64 {
    METERING.remaining_gas()
}

pub(crate) fn metering_set_data_const() -> u64 {
    METERING.set_data_const()
}

pub(crate) fn metering_set_data_key_mult() -> usize {
    METERING.set_data_key_mult()
}

pub(crate) fn metering_set_data_value_mult() -> usize {
    METERING.set_data_value_mult()
}

pub(crate) fn metering_get_data_const() -> u64 {
    METERING.get_data_const()
}

pub(crate) fn metering_get_data_key_mult() -> usize {
    METERING.get_data_key_mult()
}

pub(crate) fn metering_get_data_value_mult() -> usize {
    METERING.get_data_value_mult()
}

pub(crate) fn metering_append_data_const() -> u64 {
    METERING.append_data_const()
}

pub(crate) fn metering_append_data_key_mult() -> usize {
    METERING.append_data_key_mult()
}

pub(crate) fn metering_append_data_value_mult() -> usize {
    METERING.append_data_value_mult()
}

pub(crate) fn metering_delete_data_const() -> u64 {
    METERING.delete_data_const()
}

pub(crate) fn metering_delete_data_key_mult() -> usize {
    METERING.delete_data_key_mult()
}

pub(crate) fn metering_has_data_const() -> u64 {
    METERING.has_data_const()
}

pub(crate) fn metering_has_data_key_mult() -> usize {
    METERING.has_data_key_mult()
}

pub(crate) fn metering_local_execution_const() -> u64 {
    METERING.local_execution_const()
}

pub(crate) fn metering_local_call_const() -> u64 {
    METERING.local_call_const()
}

pub(crate) fn metering_get_bytecode_const() -> u64 {
    METERING.get_bytecode_const()
}

pub(crate) fn metering_get_bytecode_value_mult() -> usize {
    METERING.get_bytecode_value_mult()
}

pub(crate) fn metering_hash_const() -> u64 {
    METERING.get_hash_const()
}

pub(crate) fn metering_hash_per_byte() -> usize {
    METERING.hash_per_byte()
}

pub(crate) fn metering_get_owned_addrs() -> u64 {
    METERING.get_owned_addrs()
}

pub(crate) fn metering_get_call_stack() -> u64 {
    METERING.get_call_stack()
}

pub(crate) fn metering_signature_verify_const() -> u64 {
    METERING.signature_verify_const()
}

pub(crate) fn metering_signature_verify_data_mult() -> usize {
    METERING.signature_verify_data_mult()
}

pub(crate) fn metering_address_from_public_key() -> u64 {
    METERING.address_from_public_key()
}

pub(crate) fn metering_unsafe_random() -> u64 {
    METERING.unsafe_random()
}

pub(crate) fn metering_get_time() -> u64 {
    METERING.get_time()
}

pub(crate) fn metering_get_current_period() -> u64 {
    METERING.get_current_period()
}

pub(crate) fn metering_get_current_thread() -> u64 {
    METERING.get_current_thread()
}

pub(crate) fn max_number_of_pages() -> u32 {
    METERING.max_number_of_pages()
}

pub(crate) fn metering_send_message() -> u64 {
    METERING.send_message()
}

pub(crate) fn metering_set_bytecode_mult() -> usize {
    METERING.set_bytecode_mult()
}

pub(crate) fn metering_set_bytecode_const() -> u64 {
    METERING.set_bytecode_const()
}

pub(crate) fn metering_initial_cost() -> u64 {
    METERING.initial_cost()
}

pub(crate) fn max_op_datastore_entry_count() -> usize {
    METERING.max_op_datastore_entry_count()
}

pub(crate) fn has_op_key_mult() -> usize {
    METERING.has_op_key_mult()
}

pub(crate) fn get_op_data_mult() -> usize {
    METERING.get_op_data_mult()
}

pub(crate) fn get_op_keys_mult() -> usize {
    METERING.get_op_keys_mult()
}

pub(crate) fn get_keys_mult() -> usize {
    METERING.get_keys_mult()
}

pub(crate) fn max_datastore_entry_count() -> usize {
    METERING.max_datastore_entry_count()
}

#[cfg(test)]
pub(crate) fn set_metering(call_price: u64) {
    METERING._reset(call_price, DEFAULT_METERING_INITIAL_COST);
}

#[cfg(test)]
pub(crate) fn set_metering_initial_cost(initial_cost: u64) {
    METERING._reset(DEFAULT_METERING_CALL, initial_cost);
}

#[cfg(test)]
pub(crate) fn reset_metering() {
    METERING._reset(DEFAULT_METERING_CALL, DEFAULT_METERING_INITIAL_COST);
}
