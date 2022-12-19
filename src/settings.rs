// main function name in the webassembly module
pub(crate) const MAIN: &str = "main";

pub(crate) fn max_number_of_pages() -> u32 {
    64
}

pub(crate) fn max_datastore_entry_count() -> usize {
    100_000
}

pub(crate) fn max_op_datastore_entry_count() -> usize {
    128
}

#[cfg(test)]
pub(crate) fn set_metering(call_price: u64) {
    METERING._reset(call_price, 100);
}

#[cfg(test)]
pub(crate) fn set_metering_initial_cost(initial_cost: u64) {
    METERING._reset(200, initial_cost);
}

#[cfg(test)]
pub(crate) fn reset_metering() {
    METERING._reset(200, 100);
}
