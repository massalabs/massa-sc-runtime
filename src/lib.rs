#[macro_use]
extern crate lazy_static;

mod abi_impl;
mod env;
mod execution_impl;
mod settings;
mod tunable_memory;
mod types;

pub use execution_impl::{run_function, run_main};
pub use types::*;

#[cfg(test)]
mod tests;
