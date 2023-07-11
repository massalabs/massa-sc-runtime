#![feature(let_chains)]

mod error;
mod execution;
mod middlewares;
mod settings;
mod tunable_memory;
mod types;
mod wasmv1_execution;

pub use error::VMError;
pub use execution::{run_function, run_main};
pub use execution::{Compiler, RuntimeModule};
pub use types::*;

#[cfg(feature = "gas_calibration")]
pub use execution::run_main_gc;
#[cfg(feature = "gas_calibration")]
pub use middlewares::gas_calibration::GasCalibrationResult;

#[cfg(test)]
mod tests;
