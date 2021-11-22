pub type Address = u64;
pub type Bytecode = Vec::<u8>;
pub type Ledger = std::collections::BTreeMap<Address, Bytecode>; // Byttecode instead of String