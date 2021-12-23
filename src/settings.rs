use cornetto::Cornetto;

// main function name in the webassembly module
pub(crate) const MAIN: &str = "main";

// Metering private implementation
#[allow(dead_code)]
#[derive(Cornetto)]
struct Metering {
    #[cornetto(mut, 200)]
    pub call_price: u64,
}

pub(crate) fn call_price() -> u64 {
    METERING.call_price()
}

#[cfg(test)]
pub(crate) fn reset_metering(call_price: u64) {
    METERING._reset(call_price);
}
