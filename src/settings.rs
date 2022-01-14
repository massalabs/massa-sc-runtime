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
    pub has_data_const: u64,
    #[cornetto(const, 1)]
    pub has_data_key_mult: usize,
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
}

pub(crate) fn metering_call() -> u64 {
    METERING.call()
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

pub(crate) fn metering_has_data_const() -> u64 {
    METERING.has_data_const()
}

pub(crate) fn metering_has_data_key_mult() -> usize {
    METERING.has_data_key_mult()
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

#[cfg(test)]
pub(crate) fn set_metering(call_price: u64) {
    METERING._reset(call_price);
}

#[cfg(test)]
pub(crate) fn reset_metering() {
    METERING._reset(DEFAULT_METERING_CALL);
}
