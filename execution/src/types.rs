pub type Address = String;
pub type Bytecode = Vec::<u8>;
pub type Ledger = std::collections::BTreeMap<Address, Bytecode>; // Byttecode instead of String

/// That's what is returned when a module is executed correctly since the end
pub struct Response {
    /// returned value from the module call
    pub ret: String,
    /// number of points that remain after the execution (metering)
    pub remaining_points: u64,
}
