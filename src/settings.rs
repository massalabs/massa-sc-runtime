pub(crate) const MAIN: &str = "main"; // main function name in the webassembly module

// Definition of default value

/// Default price in operations of the `call` ABI
const DEFAULT_CALL_PRICE: u64 = 200;

// End of default definition

// Metering private implementation
struct PrivMetering {
    pub call_price: u64,
}

impl Default for PrivMetering {
    fn default() -> Self {
        Self {
            call_price: DEFAULT_CALL_PRICE,
        }
    }
}

#[derive(Default)]
pub(crate) struct Metering {
    p_impl: std::sync::Mutex<PrivMetering>,
}

impl Metering {
    pub fn call_price(&self) -> u64 {
        self.p_impl.lock().unwrap().call_price
    }
    #[cfg(test)]
    pub fn _reset(&self, call_price: u64) {
        self.p_impl.lock().unwrap().call_price = call_price;
    }
}

lazy_static::lazy_static! {
    pub(crate) static ref METERING: Metering = Metering::default();
}
