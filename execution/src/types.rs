pub type Address = String;
pub type Bytecode = Vec::<u8>;
pub type Ledger = std::collections::BTreeMap<Address, Bytecode>; // Byttecode instead of String