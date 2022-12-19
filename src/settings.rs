use cornetto::Cornetto;
use std::collections::HashMap;

// main function name in the webassembly module
pub(crate) const MAIN: &str = "main";

lazy_static::lazy_static!(
    pub static ref GAS_COSTS: HashMap<String, u64> = {
        let abi_costs_filename = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/gas_costs/abi_gas_costs.json"
        );
        serde_json::from_str(&std::fs::read_to_string(abi_costs_filename).unwrap()).unwrap()
    };

    pub static ref OPERATOR_COST: u64 = {
        let wasm_costs_filename = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/gas_costs/wasm_gas_costs.json"
        );
        let costs: HashMap<String, u64> = serde_json::from_str(&std::fs::read_to_string(wasm_costs_filename).unwrap()).unwrap();
        costs.iter().map(|(_, v)| v).sum::<u64>() / costs.len() as u64
    };
);

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
