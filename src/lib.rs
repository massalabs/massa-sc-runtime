#![feature(let_chains)]

mod as_execution;
mod env;
mod execution_impl;
mod middlewares;
mod settings;
mod tunable_memory;
mod types;

pub use as_execution::{Compiler, RuntimeModule};
pub use execution_impl::{run_function, run_main};
pub use types::*;

#[cfg(feature = "gas_calibration")]
pub use execution_impl::run_main_gc;
#[cfg(feature = "gas_calibration")]
pub use middlewares::gas_calibration::GasCalibrationResult;

#[cfg(test)]
mod tests;
