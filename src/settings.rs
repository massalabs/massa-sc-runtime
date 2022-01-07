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
    #[cornetto(const, 1)]
    pub set_data_mult: usize,
    #[cornetto(const, 1)]
    pub get_data_mult: usize,
    #[cornetto(const, 1)]
    pub create_sc_mult: usize,
    #[cornetto(const, 200)]
    pub print: u64,
    #[cornetto(const, 200)]
    pub remaining_gas: u64,
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

pub(crate) fn metering_set_data_mult() -> usize {
    METERING.set_data_mult()
}

pub(crate) fn metering_get_data_mult() -> usize {
    METERING.get_data_mult()
}

#[cfg(test)]
pub(crate) fn set_metering(call_price: u64) {
    METERING._reset(call_price);
}

#[cfg(test)]
pub(crate) fn reset_metering() {
    METERING._reset(DEFAULT_METERING_CALL);
}
